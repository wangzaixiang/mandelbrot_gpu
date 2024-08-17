# Mandelbrot Set Generate, CPU vs GPU

## build and running
```bash

cargo build --release

# run with cpu(1 thread)
target/release/mandlebrot cpu cpu.png 4096x2160 -1.20,0.35  -1,0.20

# run with cpu(parallel)
target/release/mandlebrot cpu_par cpu_par.png 4096x2160 -1.20,0.35  -1,0.20  

# run with gpu
target/release/mandlebrot gpu gpu.png 4096x2160 -1.20,0.35  -1,0.20  
```

## Result

| CPU(1 thread) | CPU(parallel) | GPU    |
|---------------|---------------|--------|
| 3.097s        | 0.368s        | 0.064s |
| 1x            | 8.4x          | 48.4x  |
