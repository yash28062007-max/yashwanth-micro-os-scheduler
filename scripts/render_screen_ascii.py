with open("results/screen.ppm", "rb") as f:
    f.readline()
    w, h = map(int, f.readline().decode().split())
    f.readline()
    pixels = f.read()

# Find row and col bounds of non-black pixels
rows_with_pixels = []
for y in range(h):
    row_pixels = []
    for x in range(w):
        idx = (y * w + x) * 3
        if (pixels[idx], pixels[idx+1], pixels[idx+2]) != (0, 0, 0):
            row_pixels.append(x)
    if row_pixels:
        rows_with_pixels.append((y, min(row_pixels), max(row_pixels)))

print(f"Non-black rows count: {len(rows_with_pixels)}")
if rows_with_pixels:
    min_y = rows_with_pixels[0][0]
    max_y = rows_with_pixels[-1][0]
    min_x = min(r[1] for r in rows_with_pixels)
    max_x = max(r[2] for r in rows_with_pixels)
    print(f"Bounding box: X=[{min_x}, {max_x}], Y=[{min_y}, {max_y}]")
    
    # Render downsampled ascii view:
    for y in range(min_y, max_y + 1, 2):
        line = ""
        for x in range(min_x, max_x + 1):
            idx = (y * w + x) * 3
            if (pixels[idx], pixels[idx+1], pixels[idx+2]) != (0, 0, 0):
                line += "#"
            else:
                line += " "
        print(f"{y:3d}: {line}")
