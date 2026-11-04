// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen geometry/query.ts vertex selection over the actual owned graph.
use super::*;
#[derive(Clone, Copy)]
pub(super) struct Query {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub normal: Option<([f64; 3], f64)>,
    pub limit: usize,
}
pub(super) struct Found {
    pub vertices: Vec<u32>,
    pub matched: usize,
    pub step: usize,
}
fn xyz(v: &Value) -> Result<[f64; 3], Fault> {
    let Value::Array(a) = v else {
        return Err(Fault::host("admitted vector shape"));
    };
    if a.len() != 3 {
        return Err(Fault::host("admitted vector length"));
    }
    let mut out = [0.; 3];
    for (i, v) in a.iter().enumerate() {
        let Value::Number(n) = v else {
            return Err(Fault::host("admitted vector number"));
        };
        out[i] = *n
    }
    Ok(out)
}
pub(super) fn query(graph: &MeshGraph, q: Query) -> Result<Found, KernelError> {
    if graph.vertex_count > 1_000_000 {
        return Err(KernelError::Host("native query point budget".into()));
    }
    let (mut dx, mut dy, mut dz, mut cos) = (0., 0., 0., -2.);
    if let Some((dir, c)) = q.normal {
        let len = port_core::hypot3(dir);
        if len > 0. {
            dx = dir[0] / len;
            dy = dir[1] / len;
            dz = dir[2] / len;
            cos = c
        }
    }
    let mut hits = Vec::new();
    hits.try_reserve_exact(graph.vertex_count)
        .map_err(|e| KernelError::Host(format!("{e:?}")))?;
    for v in 0..graph.vertex_count {
        let j = v * 3;
        let (x, y, z) = (
            f64::from(graph.positions[j]),
            f64::from(graph.positions[j + 1]),
            f64::from(graph.positions[j + 2]),
        );
        if x < q.min[0]
            || x > q.max[0]
            || y < q.min[1]
            || y > q.max[1]
            || z < q.min[2]
            || z > q.max[2]
        {
            continue;
        }
        if cos > -2.
            && f64::from(graph.normals[j]) * dx
                + f64::from(graph.normals[j + 1]) * dy
                + f64::from(graph.normals[j + 2]) * dz
                < cos
        {
            continue;
        }
        hits.push(v as u32)
    }
    let matched = hits.len();
    let limit = q.limit.max(1);
    if matched <= limit {
        return Ok(Found {
            vertices: hits,
            matched,
            step: 1,
        });
    }
    let step = matched.div_ceil(limit);
    let mut kept = Vec::new();
    kept.try_reserve_exact(matched.div_ceil(step))
        .map_err(|e| KernelError::Host(format!("{e:?}")))?;
    for i in (0..matched).step_by(step) {
        kept.push(hits[i])
    }
    Ok(Found {
        vertices: kept,
        matched,
        step,
    })
}
pub(super) fn box_query(
    state: &State,
    graph: Arc<MeshGraph>,
    bounds: &Value,
    normal: Option<&Value>,
) -> Result<KernelWork<Vec<u32>>, Fault> {
    let min = xyz(get(bounds, "min").ok_or_else(|| Fault::host("missing admitted box min"))?)?;
    let max = xyz(get(bounds, "max").ok_or_else(|| Fault::host("missing admitted box max"))?)?;
    for k in 0..3 {
        if min[k] > max[k] {
            return Err(command_error(
                CommandErrorCode::InvalidInput,
                "The box’s min lies past its max.",
            ));
        }
    }
    let facing = if let Some(n) = normal {
        let dir = xyz(get(n, "dir").ok_or_else(|| Fault::host("missing admitted direction"))?)?;
        if port_core::hypot3(dir) <= 0. {
            return Err(command_error(
                CommandErrorCode::InvalidInput,
                "normal.dir: give a direction, not a zero vector.",
            ));
        }
        let within = match get(n, "withinDeg") {
            Some(Value::Number(n)) => *n,
            _ => 30.,
        };
        Some((dir, ((within * std::f64::consts::PI) / 180.).cos()))
    } else {
        None
    };
    state
        .worker
        .borrow_mut()
        .reserve_request()
        .map_err(Fault::host)?;
    let q = Query {
        min,
        max,
        normal: facing,
        limit: graph.vertex_count.max(1),
    };
    start_job("scanruler-box-query", move || {
        query(&graph, q).and_then(|v| {
            if v.step != 1 || v.matched != v.vertices.len() {
                return Err(KernelError::Host(
                    "box query unexpectedly thinned its marking".into(),
                ));
            }
            Ok(v.vertices)
        })
    })
}

#[path = "native_query_definition.rs"]
mod definition;
/// The command layer's own immutable marked scope, separate from UI brush state.
pub(super) struct Marked {
    vertices: Arc<Vec<u32>>,
    from: Text,
    scan_version: u64,
}
pub(super) fn require_marked(state: &State) -> Result<Vec<u32>, Fault> {
    let current = state.worker.borrow().versions().scan;
    let mut cache = state.command_marked.borrow_mut();
    if cache.as_ref().is_some_and(|m| m.scan_version != current) {
        *cache = None
    }
    cache.as_ref().map(|m|(*m.vertices).clone()).ok_or_else(||command_error(CommandErrorCode::InvalidState,"Nothing is marked — element.fit { mark: true }, scan.segment { mark: <region id> } or scan.query { mark: true } marks points."))
}
struct Gathered {
    found: Found,
    positions: Vec<f32>,
    normals: Vec<f32>,
}
fn gather(graph: &MeshGraph, q: Query) -> Result<Gathered, KernelError> {
    let found = query(graph, q)?;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let n = found.vertices.len() * 3;
    positions
        .try_reserve_exact(n)
        .map_err(|e| KernelError::Host(format!("{e:?}")))?;
    normals
        .try_reserve_exact(n)
        .map_err(|e| KernelError::Host(format!("{e:?}")))?;
    for &v in &found.vertices {
        let j = v as usize * 3;
        positions.extend_from_slice(&graph.positions[j..j + 3]);
        normals.extend_from_slice(&graph.normals[j..j + 3]);
    }
    Ok(Gathered {
        found,
        positions,
        normals,
    })
}
fn round(n: f64) -> f64 {
    let f = n.floor();
    if n - f < 0.5 {
        f
    } else {
        let v = f + 1.;
        if v == 0. && n < 0. { -0. } else { v }
    }
}
fn decimal(n: f64, places: i32) -> Value {
    let factor = 10_f64.powi(places);
    Value::Number(round(n * factor) / factor)
}
fn triple(v: [f64; 3], places: i32) -> Value {
    Value::Array(v.into_iter().map(|n| decimal(n, places)).collect())
}
fn in_ball(q: &Gathered, at: [f64; 3], radius: f64) -> Vec<usize> {
    (0..q.found.vertices.len())
        .filter(|&i| {
            let j = i * 3;
            port_core::hypot3([
                f64::from(q.positions[j]) - at[0],
                f64::from(q.positions[j + 1]) - at[1],
                f64::from(q.positions[j + 2]) - at[2],
            ]) <= radius
        })
        .collect()
}
fn arg3(input: &Value, key: &str) -> Result<[f64; 3], Fault> {
    xyz(get(input, key).ok_or_else(|| Fault::host("missing admitted vector"))?)
}
pub(super) fn register(runtime: &Runtime, state: &Rc<State>) -> Result<(), Fault> {
    let weak = Rc::downgrade(state);
    runtime.register(
        &text("scan"),
        definition::definition(),
        Callbacks {
            label: None,
            discards: None,
            run: Rc::new(move |input, ctx| {
                let state = weak
                    .upgrade()
                    .ok_or_else(|| Fault::host("closed native query owner"))?;
                start_query(state, input, ctx)
            }),
        },
    )?;
    Ok(())
}
fn start_query(state: Rc<State>, input: &Value, ctx: CommandContext) -> Result<(), Fault> {
    let app = state.document.store(StoreId::App);
    if matches!(app.get("fileName"), C::Null | C::Undefined)
        || matches!(app.get("vertexCount"), C::Number(0.))
    {
        return Err(command_error(
            CommandErrorCode::NoScan,
            "No scan is open — open one with scan.open first.",
        ));
    }
    let boxed = get(input, "box");
    let near = get(input, "near");
    if boxed.is_some() == near.is_some() {
        return Err(command_error(
            CommandErrorCode::InvalidInput,
            "Give box or near — one of them.",
        ));
    }
    let ball = if let Some(near) = near {
        let at = arg3(near, "at")?;
        let radius = match get(near, "radius") {
            Some(Value::Number(n)) => *n,
            _ => return Err(Fault::host("admitted ball radius")),
        };
        Some((at, radius))
    } else {
        None
    };
    let (min, max) = if let Some(b) = boxed {
        (arg3(b, "min")?, arg3(b, "max")?)
    } else {
        let (at, r) = ball.ok_or_else(|| Fault::host("missing query ball"))?;
        (at.map(|x| x - r), at.map(|x| x + r))
    };
    for k in 0..3 {
        if min[k] > max[k] {
            return Err(command_error(
                CommandErrorCode::InvalidInput,
                "The box’s min lies past its max.",
            ));
        }
    }
    let normal = if let Some(n) = get(input, "normal") {
        let dir = arg3(n, "dir")?;
        let degrees = match get(n, "withinDeg") {
            Some(Value::Number(n)) => *n,
            _ => 30.,
        };
        Some((dir, ((degrees * std::f64::consts::PI) / 180.).cos()))
    } else {
        None
    };
    let limit = match get(input, "limit") {
        Some(Value::Number(n)) => *n as usize,
        _ => 1000,
    };
    let mark = matches!(get(input, "mark"), Some(Value::Boolean(true)));
    let fields = match get(input, "fields") {
        Some(Value::String(s)) => String::from_utf16_lossy(s),
        _ => {
            if mark {
                "count".into()
            } else {
                "all".into()
            }
        }
    };
    let graph = state
        .worker
        .borrow()
        .graph()
        .ok_or_else(|| Fault::host("missing query graph"))?;
    let q = Query {
        min,
        max,
        normal,
        limit: if ball.is_some() {
            20000.min((limit as f64 * 1.91).ceil() as usize)
        } else {
            limit
        },
    };
    state
        .worker
        .borrow_mut()
        .reserve_request()
        .map_err(Fault::host)?;
    let worker_graph = graph.clone();
    let work = start_job("scanruler-query", move || gather(&worker_graph, q))?;
    ctx.spawn(finish_query(
        state,
        work,
        QueryInput {
            graph,
            q,
            ball,
            mark,
            fields,
            limit,
        },
    ))?;
    Ok(())
}
struct QueryInput {
    graph: Arc<MeshGraph>,
    q: Query,
    ball: Option<([f64; 3], f64)>,
    mark: bool,
    fields: String,
    limit: usize,
}
async fn finish_query(
    state: Rc<State>,
    work: KernelWork<Gathered>,
    input: QueryInput,
) -> Result<Value, Fault> {
    let QueryInput {
        graph,
        q,
        ball,
        mark,
        fields,
        limit,
    } = input;
    let r = work.await.map_err(KernelError::fault)?;
    let every = if mark {
        let all = if r.found.step > 1 {
            state
                .worker
                .borrow_mut()
                .reserve_request()
                .map_err(Fault::host)?;
            let worker_graph = graph.clone();
            let full = Query {
                limit: graph.vertex_count.max(1),
                ..q
            };
            Some(
                start_job("scanruler-query-mark", move || gather(&worker_graph, full))?
                    .await
                    .map_err(KernelError::fault)?,
            )
        } else {
            None
        };
        let all = all.as_ref().unwrap_or(&r);
        Some(if let Some((at, radius)) = ball {
            in_ball(all, at, radius)
                .into_iter()
                .map(|i| all.found.vertices[i])
                .collect::<Vec<_>>()
        } else {
            all.found.vertices.clone()
        })
    } else {
        None
    };
    let mut matched = r.found.matched;
    let mut vertices = r.found.vertices.clone();
    let mut positions = r.positions.clone();
    let mut normals = r.normals.clone();
    if let Some((at, radius)) = ball {
        let keep = in_ball(&r, at, radius);
        matched = if let Some(every) = &every {
            every.len()
        } else if r.found.step > 1 {
            round(
                (keep.len() as f64 / r.found.vertices.len().max(1) as f64) * r.found.matched as f64,
            ) as usize
        } else {
            keep.len()
        };
        let stride = keep.len().div_ceil(limit).max(1);
        let kept = keep.into_iter().step_by(stride).collect::<Vec<_>>();
        vertices = kept.iter().map(|&i| r.found.vertices[i]).collect();
        positions = kept
            .iter()
            .flat_map(|&i| r.positions[i * 3..i * 3 + 3].iter().copied())
            .collect();
        normals = kept
            .iter()
            .flat_map(|&i| r.normals[i * 3..i * 3 + 3].iter().copied())
            .collect();
    }
    let n = vertices.len();
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    let mut centroid = [0.; 3];
    let mut mean_normal = [0.; 3];
    for i in 0..n {
        for k in 0..3 {
            let v = f64::from(positions[i * 3 + k]);
            if v < lo[k] {
                lo[k] = v
            }
            if v > hi[k] {
                hi[k] = v
            }
            centroid[k] += v / n as f64;
            mean_normal[k] += f64::from(normals[i * 3 + k]);
        }
    }
    let nl = port_core::hypot3(mean_normal);
    let marked = if let Some(every) = every {
        let from = text(&format!(
            "scan.query {}{}",
            if ball.is_some() { "ball" } else { "box" },
            if q.normal.is_some() {
                " facing a way"
            } else {
                ""
            }
        ));
        let marked = Marked {
            vertices: Arc::new(every),
            from,
            // Source setMarked reads the current client generation at completion.
            scan_version: state.worker.borrow().versions().scan,
        };
        let summary = object(vec![
            ("points", Value::Number(marked.vertices.len() as f64)),
            ("from", Value::String(marked.from.clone())),
        ]);
        *state.command_marked.borrow_mut() = Some(marked);
        Some(summary)
    } else {
        None
    };
    let step = if ball.is_some() {
        round(matched as f64 / n.max(1) as f64).max(1.) as usize
    } else {
        r.found.step
    };
    let mut out = vec![
        ("matched", Value::Number(matched as f64)),
        ("returned", Value::Number(n as f64)),
        ("step", Value::Number(step as f64)),
    ];
    if n > 0 {
        out.push((
            "bounds",
            object(vec![("min", triple(lo, 4)), ("max", triple(hi, 4))]),
        ));
        out.push(("centroid", triple(centroid, 4)));
        out.push((
            "meanNormal",
            if nl > 0. {
                triple(mean_normal.map(|v| v / nl), 4)
            } else {
                Value::Null
            },
        ));
    }
    if let Some(marked) = marked {
        out.push(("marked", marked))
    }
    if fields != "count" {
        out.push((
            "vertices",
            Value::Array(
                vertices
                    .into_iter()
                    .map(|v| Value::Number(f64::from(v)))
                    .collect(),
            ),
        ))
    }
    if fields == "all" {
        let triples = |a: Vec<f32>, p| {
            Value::Array(
                a.as_chunks::<3>()
                    .0
                    .iter()
                    .map(|v| triple([f64::from(v[0]), f64::from(v[1]), f64::from(v[2])], p))
                    .collect(),
            )
        };
        out.push(("points", triples(positions, 4)));
        out.push(("normals", triples(normals, 3)));
    }
    Ok(object(out))
}
