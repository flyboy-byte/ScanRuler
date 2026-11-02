# Independent image inputs — 2026-11-02

56 authored files, seeds 1/7/99/1039, AGPL-3.0-only. No downloaded/private images.
Pillow 13.0.0/reference encoders generate these inputs from authored pixels;
Pillow is never an application decoder. Manifest records encoder versions,
formats, case IDs, byte counts and hashes. Rust outputs are not fixture inputs
or expected values. Do not regenerate these files to make a candidate pass.

PNG covers an entire 256×256 color/alpha atlas, opaque RGB, grayscale/LA,
16-bit grayscale, palette transparency and a gamma chunk. JPEG covers two
qualities, two subsampling choices, progressive/grayscale/CMYK and seeded inputs.
WebP includes lossless and lossy alpha; BMP includes RGB/RGBA and seeded inputs.
Truncations, invalid signatures and junk exercise source omission separately.
Animation, Exif orientation, ICC/other profiles, interlacing and rare variants
remain open; a format label is not complete format acceptance.

Full original Chrome pixels/omissions are frozen in image-source.json before
candidate decoding; candidate frames retain every mismatch. See the codec audit
finding and decision. These are audit observations, not working import commands
or additional successful native numerical cases in the application gate.
