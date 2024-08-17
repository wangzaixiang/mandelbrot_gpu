
running `mandlebrot output.jpg 4096x2160 -1.20,0.35  -1,0.20` 
1. cpu 1 threads: 2.665s
2. into par: 0.353s, 7.5x faster
3. write a gpu version using wgpu-rs