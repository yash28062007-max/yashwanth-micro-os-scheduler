import os

ppm_path = "results/screen3.ppm"
if not os.path.exists(ppm_path):
    print("No screen3.ppm found")
    exit(1)

with open(ppm_path, "rb") as f:
    magic = f.readline().decode().strip()
    dims = f.readline().decode().strip().split()
    w, h = int(dims[0]), int(dims[1])
    maxval = int(f.readline().decode().strip())
    raw_pixels = f.read()

print(f"PPM {w}x{h}, {len(raw_pixels)} bytes")
# Sample colors:
colors = {}
for i in range(0, len(raw_pixels), 3):
    rgb = (raw_pixels[i], raw_pixels[i+1], raw_pixels[i+2])
    colors[rgb] = colors.get(rgb, 0) + 1

print(f"Found {len(colors)} distinct colors.")
sorted_colors = sorted(colors.items(), key=lambda x: x[1], reverse=True)
for c, cnt in sorted_colors[:10]:
    print(f"Color {c}: {cnt} pixels ({cnt*100/(w*h):.1f}%)")
