Alright — let’s build the **“proper” seekable HTTP audio pipeline** using:

* reqwest → HTTP + range requests
* symphonia → container parsing + accurate seeking

This is the approach where **the decoder drives the seek**, not you guessing byte offsets.

---

# 🧠 Architecture Overview

Instead of:

> “time → byte offset → HTTP”

We do:

> decoder parses file → builds seek index → asks us to seek → we fetch correct byte range

So your job is to implement a **custom media source** that:

✔ supports `Read + Seek`
✔ performs HTTP range requests internally
✔ is *transparent* to the decoder

---

# 🧱 Step 1: Define the HTTP-backed source

```rust
use std::io::{Read, Seek, SeekFrom, Result};
use reqwest::blocking::Client;

pub struct HttpMediaSource {
    url: String,
    client: Client,
    pos: u64,
    content_length: Option<u64>,

    // current HTTP response (range)
    buffer: Vec<u8>,
    buffer_start: u64,
}
```

---

# 🌐 Step 2: Range request helper

```rust
impl HttpMediaSource {
    fn fetch_range(&mut self, start: u64) -> Result<()> {
        let resp = self.client
            .get(&self.url)
            .header("Range", format!("bytes={}-", start))
            .send()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        self.buffer = resp.bytes()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            .to_vec();

        self.buffer_start = start;
        Ok(())
    }
}
```

👉 In a real implementation, you’d fetch **chunks**, not the whole remainder.

---

# 📖 Step 3: Implement `Read`

```rust
impl Read for HttpMediaSource {
    fn read(&mut self, out: &mut [u8]) -> Result<usize> {
        // If buffer doesn't contain current position → fetch
        if self.pos < self.buffer_start ||
           self.pos >= self.buffer_start + self.buffer.len() as u64 {
            self.fetch_range(self.pos)?;
        }

        let offset = (self.pos - self.buffer_start) as usize;
        let available = &self.buffer[offset..];

        let n = available.len().min(out.len());
        out[..n].copy_from_slice(&available[..n]);

        self.pos += n as u64;
        Ok(n)
    }
}
```

---

# 🔁 Step 4: Implement `Seek`

```rust
impl Seek for HttpMediaSource {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(n) => n,
            SeekFrom::Current(n) => (self.pos as i64 + n) as u64,
            SeekFrom::End(n) => {
                let len = self.content_length
                    .ok_or_else(|| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "unknown length"
                    ))?;
                (len as i64 + n) as u64
            }
        };

        self.pos = new_pos;
        Ok(new_pos)
    }
}
```

👉 Important: **no HTTP request happens here** — just updating position.

---

# 🧩 Step 5: Hook into Symphonia

`symphonia` expects a `MediaSource`.

```rust
use symphonia::core::io::MediaSource;

impl MediaSource for HttpMediaSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.content_length
    }
}
```

---

# 🎵 Step 6: Create the decoder

```rust
use symphonia::default::get_probe;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::io::MediaSourceStream;

let source = HttpMediaSource::new(url);
let mss = MediaSourceStream::new(Box::new(source), Default::default());

let probed = get_probe().format(
    &Default::default(),
    mss,
    &FormatOptions::default(),
    &MetadataOptions::default(),
)?;
```

---

# ⏱️ Step 7: Perform accurate seeking

Now the magic:

```rust
use symphonia::core::formats::SeekMode;
use symphonia::core::formats::SeekTo;

let format = probed.format;

format.seek(
    SeekMode::Accurate,
    SeekTo::Time {
        time: 83.0.into(), // seconds
        track_id: None,
    },
)?;
```

👉 What happens internally:

1. `symphonia` looks at container metadata
2. Computes correct byte/frame position
3. Calls your `Seek` implementation
4. Your next `Read` triggers an HTTP range request

✔ This is *true format-aware seeking*

---

# 🚧 Step 8: Make it actually robust

The above is minimal. A real implementation needs:

---

## 📦 Chunked buffering

Instead of downloading everything after `start`:

```rust
Range: bytes=123456-131072
```

Use fixed-size windows (e.g. 64KB–1MB)

---

## 🔁 Readahead

Prefetch next chunk to avoid playback stalls.

---

## 🧠 Smart caching

Keep:

* current chunk
* maybe previous chunk

---

## ⚠️ Retry logic

Handle:

* network drops
* partial reads
* timeouts

---

## 📊 Initial probe

Before playback:

* fetch first ~64KB
* extract:

    * duration
    * seek tables (Xing, etc.)

---

# ⚖️ Why This Works

Because symphonia:

✔ understands MP3, AAC, FLAC containers
✔ parses seek tables
✔ can fall back to frame scanning

So instead of guessing:

> “1:23 ≈ byte 1.2MB”

It knows:

> “1:23 is exactly frame X at byte Y”

---

# 🧠 Mental Model

Your `HttpMediaSource` is basically:

> A **fake file** backed by HTTP

And `symphonia` treats it like a normal file.

---

# 🔥 Key Insight

The hardest part is NOT seeking — it’s:

> Making HTTP behave like a lazy, partial file system

Once you do that, the decoder handles everything else.

---

# 🚀 If You Want to Go Further

Next steps I can show you:

* 🔊 Plug this into actual playback (e.g. rodio or CPAL)
* ⚡ Async version using `tokio`
* 📡 Add caching layer (almost like a mini streaming engine)
* 🎧 Support for live streams / partial content

Just tell me what direction you want 👍
