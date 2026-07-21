#!/usr/bin/env python3
"""Replace the ASCII terminal mockup in index.html (zh + en) with the real
rendered PNG screenshot. Keep the wrapper div for layout + rise animation."""
import re
import sys

NEW_BLOCK = '''<div class="demo-wrap rise d4">
  <img src="assets/angles-demo.png" alt="angles terminal session" loading="lazy" width="1180" height="722" style="width:100%;height:auto;display:block;border-radius:16px;border:1px solid rgba(0,0,0,.05);box-shadow:0 30px 80px rgba(20,18,11,.18),0 4px 16px rgba(20,18,11,.08)" />
</div>

</section>

'''

for path in sys.argv[1:]:
    with open(path, 'r', encoding='utf-8') as f:
        s = f.read()

    # Replace from <div class="demo-wrap rise d4"> up to (but not including)
    # the <section class="manifesto"> block.
    pat = re.compile(
        r'<div class="demo-wrap rise d4">.*?(?=<section class="manifesto">)',
        flags=re.DOTALL,
    )
    s2, n = pat.subn(NEW_BLOCK, s, count=1)
    if n != 1:
        print(f"⚠️  {path}: pattern not matched ({n} replacements)")
        continue

    # Also strip the now-orphaned </section> that closed the hero
    # (we already added one in NEW_BLOCK, so delete the next stray one right
    # before the manifesto section opener — that section's own closer remains).
    s2 = s2.replace('</section>\n\n\n<section class="manifesto">', '\n<section class="manifesto">', 1)

    with open(path, 'w', encoding='utf-8') as f:
        f.write(s2)
    print(f"✅ {path}: replaced demo mockup with PNG ({len(s)} → {len(s2)} bytes)")
