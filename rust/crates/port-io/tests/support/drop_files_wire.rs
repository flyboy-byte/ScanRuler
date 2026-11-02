// SPDX-License-Identifier: AGPL-3.0-only
// Test-only authored snapshot framing and immediately-ready async platform.
use port_io::drop_files::*;
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};
type Error = (String, String);
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let b = &self.b[self.at..self.at + n];
        self.at += n;
        b
    }
    fn byte(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn word(&mut self) -> usize {
        u32::from_le_bytes(self.take(4).try_into().unwrap()) as usize
    }
    fn text(&mut self) -> String {
        let n = self.word();
        String::from_utf8(self.take(n).to_vec()).unwrap()
    }
    fn error(&mut self) -> Option<Error> {
        if self.byte() == 0 {
            None
        } else {
            Some((self.text(), self.text()))
        }
    }
    fn ids(&mut self) -> Vec<usize> {
        let n = self.word();
        (0..n).map(|_| self.word()).collect()
    }
}
struct Item {
    file: bool,
    api: bool,
    entry: usize,
    error: Option<Error>,
}
struct Batch {
    error: Option<Error>,
    ids: Vec<usize>,
}
struct Entry {
    flags: u8,
    file: u32,
    fe: Option<Error>,
    re: Option<Error>,
    batches: Vec<Batch>,
}
pub struct Source {
    items: Vec<Item>,
    entries: Vec<Entry>,
    readers: Vec<(usize, usize)>,
    pub log: Vec<String>,
    pub pending: bool,
    pub infinite: bool,
}
impl Source {
    fn entry_value(&self, id: usize) -> DropEntry {
        let e = &self.entries[id];
        DropEntry {
            handle: id,
            is_file: e.flags & 1 != 0,
            is_directory: e.flags & 2 != 0,
        }
    }
}
impl DropSource for Source {
    type File = u32;
    type Error = Error;
    fn entry(&mut self, id: usize) -> Result<Option<DropEntry>, Error> {
        let i = &self.items[id];
        if !i.api {
            return Ok(None);
        }
        self.log.push(format!("entry:{id}"));
        if let Some(e) = &i.error {
            return Err(e.clone());
        }
        Ok(if i.entry == u32::MAX as usize {
            None
        } else {
            Some(self.entry_value(i.entry))
        })
    }
    fn reader(&mut self, e: DropEntry) -> Result<usize, Error> {
        self.log.push(format!("reader:{}", e.handle));
        if let Some(e) = &self.entries[e.handle].re {
            return Err(e.clone());
        }
        let id = self.readers.len();
        self.readers.push((e.handle, 0));
        Ok(id)
    }
    async fn batch(&mut self, r: usize) -> Result<Vec<DropEntry>, Error> {
        let (id, cursor) = self.readers[r];
        self.log.push(format!("batch:{id}:{cursor}"));
        self.readers[r].1 += 1;
        if self.infinite {
            return Ok(vec![self.entry_value(id)]);
        }
        let Some(b) = self.entries[id].batches.get(cursor) else {
            return Ok(Vec::new());
        };
        if let Some(e) = &b.error {
            return Err(e.clone());
        }
        Ok(b.ids.iter().map(|&i| self.entry_value(i)).collect())
    }
    async fn file(&mut self, e: DropEntry) -> Result<u32, Error> {
        self.log.push(format!("file:{}", e.handle));
        if self.pending {
            let mut ready = false;
            std::future::poll_fn(|cx| {
                if ready {
                    Poll::Ready(())
                } else {
                    ready = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            })
            .await;
        }
        let entry = &self.entries[e.handle];
        if let Some(e) = &entry.fe {
            return Err(e.clone());
        }
        Ok(entry.file)
    }
}
pub fn limits() -> DropLimits {
    DropLimits {
        max_items: 64,
        max_entries: 512,
        max_files: 256,
        max_batches: 64,
        max_calls: 1024,
    }
}
pub fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = pin!(f);
    let mut cx = Context::from_waker(Waker::noop());
    for _ in 0..10000 {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
    }
    panic!("unexpected unbounded fixture future")
}
pub fn parse(input: &[u8]) -> (Option<DropData<u32>>, Source) {
    let mut r = Reader { b: input, at: 0 };
    let present = r.byte() != 0;
    let _files = r.word();
    let files = r.ids().into_iter().map(|i| i as u32).collect();
    let n = r.word();
    let items: Vec<_> = (0..n)
        .map(|_| Item {
            file: r.byte() != 0,
            api: r.byte() != 0,
            entry: r.word(),
            error: r.error(),
        })
        .collect();
    let n = r.word();
    let entries = (0..n)
        .map(|_| {
            let flags = r.byte();
            let file = r.word() as u32;
            let fe = r.error();
            let re = r.error();
            let n = r.word();
            let batches = (0..n)
                .map(|_| Batch {
                    error: r.error(),
                    ids: r.ids(),
                })
                .collect();
            Entry {
                flags,
                file,
                fe,
                re,
                batches,
            }
        })
        .collect();
    let data = DropData {
        files,
        items: items
            .iter()
            .enumerate()
            .map(|(handle, i)| DropItem {
                handle,
                is_file: i.file,
            })
            .collect(),
    };
    assert_eq!(r.at, input.len());
    (
        present.then_some(data),
        Source {
            items,
            entries,
            readers: Vec::new(),
            log: Vec::new(),
            pending: false,
            infinite: false,
        },
    )
}
fn word(b: &mut Vec<u8>, n: usize) {
    b.extend((n as u32).to_le_bytes())
}
fn text(b: &mut Vec<u8>, s: &str) {
    word(b, s.len());
    b.extend(s.as_bytes())
}
pub fn run(input: &[u8]) -> Vec<u8> {
    let (data, mut source) = parse(input);
    let result = block_on(dropped_files(data.as_ref(), &mut source, limits()));
    let mut out = b"DRP1".to_vec();
    match result {
        Ok(ids) => {
            out.push(0);
            word(&mut out, ids.len());
            for id in ids {
                word(&mut out, id as usize)
            }
        }
        Err(DropError::Platform((name, message))) => {
            out.push(1);
            text(&mut out, &name);
            text(&mut out, &message)
        }
        other => panic!("host failure cannot become source output: {other:?}"),
    };
    word(&mut out, source.log.len());
    for s in source.log {
        text(&mut out, &s)
    }
    out
}
