set nfacs [ gtkwave::getNumFacs]
set all_facs [list]
for {set i 0} {$i < $nfacs} {incr i} {
    set facname [gtkwave::getFacName $i ]
    lappend all_facs "$facname"
}
set num_added [ gtkwave::addSignalsFromList $all_facs ]
puts "num signals added: $num_added"
gtkwave::/Time/Zoom/Zoom_Full
set dumpname [gtkwave::getDumpFileName]
gtkwave::/File/Print_To_File PDF {Letter (8.5" x 11")} Minimal $dumpname.pdf
