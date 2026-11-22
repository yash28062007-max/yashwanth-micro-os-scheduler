with open("results/screen.ppm", "rb") as f:
    f.readline()
    w, h = map(int, f.readline().decode().split())
    f.readline()
    pixels = f.read()

# Each char in 640x480 standard VGA mode 3 is 8x16 or 9x16.
# Let's extract columns
cols = []
for x in range(140, 500):
    col = []
    for y in range(224, 240):
        idx = (y * w + x) * 3
        col.append('#' if (pixels[idx], pixels[idx+1], pixels[idx+2]) != (0, 0, 0) else ' ')
    cols.append(''.join(col))

# Reconstruct 16-row text line
for y in range(16):
    row_chars = [cols[x][y] for x in range(len(cols))]
    print(f"{y:2d}: " + "".join(row_chars))
