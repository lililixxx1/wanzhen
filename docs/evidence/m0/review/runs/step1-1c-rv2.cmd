# CMD: ./target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 1
# RUN_UTC: 2026-10-06T02:52:12Z
# EXPECT: stdout last line final_hash=0x564cf46fdf191710 (t009/summary.md:245-247 golden cross; no bytewise archive - hash anchor)
# REAL_EXIT: 0
## ANCHOR
final_hash=0x564cf46fdf191710
# REAL_EXIT(grep): 0
## FRESH STDOUT (for self-containment)
seed=3000000
comp_red=shieldman:5000
comp_blue=shieldman:5000
winner=red
end_tick=14400
alive_red=4990
alive_blue=4990
final_hash=0x564cf46fdf191710
