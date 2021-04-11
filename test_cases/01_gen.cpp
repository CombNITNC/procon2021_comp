#include <fstream>
#include <iostream>

int main() {
  using ios = std::ios_base;
  std::ofstream img("01_q.ppm", ios::out | ios::binary | ios::trunc);
  if (!img) {
    std::cerr << "Failed to create 01_q.ppm\n";
    return 1;
  }
  // Generate 32x32 monochrome circle but y-shifted on right side
  img << "P6\n"
      << "# 2 2\n"
      << "# 1\n"
      << "# 3 1\n"
      << "32 32\n"
      << "255\n";
  for (int y = -16; y < 16; ++y) {
    for (int x = -16; x < 16; ++x) {
      unsigned char c;
      if (x < 0) {
        c = (x * x + y * y < 16 * 16) ? 255 : 0;
      } else {
        if (y < 0) {
          c = (x * x + (y + 16) * (y + 16) < 16 * 16) ? 255 : 0;
        } else {
          c = (x * x + (y - 16) * (y - 16) < 16 * 16) ? 255 : 0;
        }
      }
      unsigned char buf[] = {c, c, c};
      img.write(reinterpret_cast<char *>(buf), sizeof(buf) / sizeof(buf[0]));
    }
  }
  return 0;
}
