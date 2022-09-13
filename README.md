# Synopsis
This provides a cli to trigger inquire, allowing for any language with sys call abilities to run nice interactive inquiries.

# example of Yes or No question
inquire-cli -o ./answer_file.yml -c '[{"name":"test", "type":"confirm", "message":"Are you from Mars?","help":"Some help message", "placeholder": "some placeholder"}]'

# example of open-ended question
inquire-cli -o ./answer_file.yml -c '[{"name":"test", "type":"text",    "message":"Where are you from?", "suggestions":["Colombia", "Brazil", "Argentina", "USA"] }]' 


