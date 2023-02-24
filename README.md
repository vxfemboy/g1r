# G.1.R - GARBAGE INTERNET ROBOT
### GREETINGS HUMANS! 
#### Behold G.1.R, the modular IRC bot written in Rust, programmed to be your obedient servant in all your trolling and memeing needs!

```
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣴⣿⣿⠆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⠶⣻⡿⠛⠁⠀⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⣠⣤⣤⣤⡤⠤⢤⣤⣤⣤⣤⣀⣀⣠⡶⢋⣡⡾⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢰⡿⠛⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠉⠛⠻⠿⣤⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠳⣦⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠒⠀⠙⢷⣤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡌⠀⠀⠀⠁⢙⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁⠀⠀⠀⠀⣸⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⢀⡀⠀⠀⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⠀⠀⠀⠀⠀⠉⠀⠀⠀⠀⢀⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢀⣾⠛⠻⣿⢶⣤⣀⣿⠀⠀⠀⠀⠀⠀⠀⣀⡴⠖⠋⠉⠉⠉⠛⠳⣦⡀⠀⠀⠀⠀⠀⠀⣼⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⣸⠇⠀⠀⠹⡆⠘⡟⠻⣶⣄⠀⠀⠀⢀⡼⠋⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⣆⠀⠀⠀⠀⢰⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⣿⠀⠀⠀⠀⢧⠀⢿⡀⢹⠈⣧⠀⠀⡼⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⡄⠀⠀⠀⣾⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⣿⠀⠀⠀⠀⢸⠀⢸⠇⣾⠀⡿⠀⢀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡇⠀⠀⣸⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢻⡆⠀⠀⠀⣼⠀⡿⢠⡏⣸⠃⠀⠀⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣁⡄⢠⣿⣄⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠈⢷⡀⠀⣰⠃⣸⣣⡾⣿⠁⠀⠀⠀⠘⢧⡀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡼⠛⠁⠀⣼⣷⣾⣭⣭⣛⡓⠶⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠈⠻⠾⠿⠛⠉⠁⠀⠸⣧⡀⠀⠀⠀⠈⠙⢦⣀⡀⠀⠀⠀⣀⡴⠋⠀⠀⠀⣰⣿⣿⡏⢩⠉⢻⣿⣿⣶⣬⣙⠳⣦⣀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠳⣄⡀⠀⠰⠖⠋⠉⠉⠉⠉⡉⠁⠀⠀⠀⠀⣴⣿⣿⣿⣷⣾⣤⣿⣿⣿⡿⠿⠛⠃⠀⢻⡆⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠛⠦⣄⣀⡀⠀⠀⠀⠀⠙⠀⠀⣀⣴⣿⣿⣿⣿⡿⠿⠛⠛⠉⠁⠀⠀⠀⠀⢀⣠⠼⠷⠤⢤⣄⡀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢿⣏⠙⢿⣶⣶⣟⣹⡿⠿⠛⠛⠉⢁⣀⣀⣀⣀⡤⠀⠀⠀⠀⠀⡴⠋⠀⠀⠀⠀⠀⠈⠻⣆
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣽⡷⢾⣿⡯⠽⠶⢶⠒⠛⠛⣿⠛⠉⠉⠉⠉⠻⡀⠀⠀⠀⣼⠁⠀⠀⠀⠀⠀⠀⠀⠀⢻
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⢞⡁⢀⡼⢹⡇⠀⢀⣿⣦⡀⢰⡏⠀⠀⠀⠀⠀⠀⠙⡆⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠃⠀⣿
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⠻⢧⣿⣣⡟⡗⢸⠀⠀⢸⣿⣿⣿⣾⠀⠀⠀⠀⠀⠀⠀⠀⠘⠀⠀⢻⡀⠀⠀⠀⠀⠀⠀⢀⣾⠃
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⠃⢹⢷⠃⢸⠀⠀⢸⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⢠⠇⠀⠀⠀⠙⠦⣄⣀⣀⣤⡶⠟⠁⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⠇⠀⢸⣀⡇⢸⠀⠀⠸⣿⣿⣿⣽⠀⠀⠀⠀⠀⠀⣠⠏⠀⠀⠀⠀⠀⠀⠀⠀⣸⡟⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢿⡀⠀⢸⣧⡀⢸⠀⠀⠀⠈⠛⠋⣿⠀⣠⡾⠁⠀⡰⠃⠀⠀⠀⠀⠀⠀⠀⠀⢠⣿⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⣤⣘⠛⠛⠋⠀⠀⠀⠀⠀⠀⣿⡾⠋⠀⠀⠘⠁⠀⠀⠀⠀⠀⠀⠀⠀⣀⣾⠃⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢻⣿⣿⣿⡟⠒⣶⣶⣤⣤⣿⠁⠀⠀⠀⠀⠀⠀⠀⢀⣀⣠⣤⠶⠟⠛⠁⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢿⣿⣿⣷⠀⠸⣿⣿⣿⣯⠙⠛⠲⠶⠶⠶⣾⣟⠛⠉⠁⠀⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢿⣿⡿⠀⠀⠙⣿⣿⣿⠀⠀⠀⠀⠀⠀⠙⠿⠿⠶⠶⠶⠶⠶⠾⠟⠋⠀⠀⠀⠀⠀
```

this is the dev branch

## FEATURES

- OpenAI GPT-3 integration: mention G.1.R and he'll respond with AI-generated wisdom (or nonsense, depending on how you see it).
- Zim-approved: our resident robot has been certified by the Almighty Tallest to bring chaos and destruction to the IRC world.

### COMMANDS AND USAGE

- `%ping`: Send this command to G.1.R and he'll respond back with "pong" and the time it took. Because why not?
- `%kill`: Do you hate your bot companion and want to end his miserable existence? Just type this command and he'll self-destruct, leaving you alone with your conscience (and your loneliness).
- `%invade 5 #channel SCREAM`: Do you want to summon the Invaders to a specific channel? This command will do just that, plus a chance to let out your deepest screams of despair. 5 being the number of invaders to join!
- `%%scream #channel SCREAM`: In case the invaders weren't enough to scare off your enemies, use this command to make them hear your battle cry across the IRC lands.
- `%%join #channel`: More bots, more fun! Tell G.1.R to join the party in a specific channel and watch the madness unfold.
- `%%leave #channel`: Tired of all the chaos? Use this command to make the bots leave the channel and leave you in peace (or in silence, at least).

Pro tip: When you use the `%invade #channel` command, G.1.R will set up that channel to be the commander channel, so you can control all the bots from one place and only that place. Talk about efficient invasion tactics!

## TODO

- [ ] Multi-server invasion: Because why limit our chaos to one server?
- [ ] Proxy support: Because we like to cover our tracks (and we're just a little paranoid).
- [ ] Random IDents: Because who doesn't like to be mysterious?
- [ ] Bridging: To connect different IRC servers and make them fight against each other (because why not?)
- [ ] User logs: To keep track of everyone who dares to challenge the Invaders.
- [ ] User cloning: To create a legion of evil twins that will make everyone's life miserable (or just ours).
- [ ] Console: Because we like to be fancy.
- [ ] Scanner: To find more things to invade and conquer (or just to mess with).
- [x] Colors: To make our messages look pretty (because aesthetics matter).
- [x] Multi-bot: Because one bot is never enough.
- [ ] Invader arguments: Because we like to be flexible and customizable.
- [ ] Chat emulators: To trick our enemies into thinking they're talking to real humans (suckers!).
- [X] Random nicknames for invaders: Because who wants to be called "Bot 1" forever?
- [ ] DM commander: To control the bots from the shadows, like a true mastermind.
- [ ] Key rotation: To avoid API attempts from stopping the invasion.

So what are you waiting for? Join us in our quest for world domination, and let's make Invader Zim proud!