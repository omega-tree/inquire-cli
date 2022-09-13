use strict;
use warnings;


my $cmd = <<'EOF'; 
    /home/flopes/projects/inquire-cli/target/debug/inquire-cli  -o /home/flopes/answers.yml -c '[
    {"name":"test", "type":"multi_select",    "message":"Please pick a some food", "help" : "extra_help", "options":["pasta", "pizza", "meat_balls"], },
    {"name":"test2", "type":"multi_select",    "message":"Please pick a some drink", "help" : "extra_help", "options":["water", "pepsi", "cola"], }
    ]';
EOF

system($cmd);
# open(my $fh, "-|", $cmd) or die("Can't run $cmd.$!");



1;
