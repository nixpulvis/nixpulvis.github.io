set terminal png
set output 'img/ramblings/current-mirror/pnp_plot.png'
set datafile separator ','
set key autotitle columnhead
plot '_data/ramblings/current-mirror/pnp.csv' using 2:($3/$2*1000):1 with labels
