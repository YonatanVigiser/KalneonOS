set print pretty on
set history save on
set history filename .gdb_history
set history size 1000
set logging file logs/gdb.log
set logging overwrite on
set logging enabled on
set tui mouse-events off
tui new-layout kdbg src 1 status 0 cmd 2
layout kdbg
focus cmd
target remote localhost:26000
hbreak main
