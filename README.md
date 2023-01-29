# how-active
`how-active` is a terminal UI-based tool for analysing a user's activity on Discord.
Input a USER ID (enable Developer's mode in Discord, right-click on a user and select `Copy ID`)
and a GUILD or CHANNEL ID (obtained the same way, except by right-clicking on a server or channel).

After verifying your target user and target channel/guild in the top right window, 
you may hit the `s` key to start the procedure. `how-active` will use the Discord 
search API to grab the messages sent by the given user and will parse the timestamp 
to increment that given hour in the chart.

## Instructions
1. Run the program through the terminal `./how-active`, providing a TOKEN as an argument or
writing it to an environment variable `DISCORD_TOKEN`. If you have the token in your
environment variables, you may double click the executable to open it in a terminal.
2. In the terminal hit `i` to start entering a target USER ID. Submit the ID by hitting `ENTER`
3. Hit `a` to enter the CHANNEL or GUILD ID. Once again submit with `ENTER`
4. The `s` key starts the process.
5. `q` is at your disposal to quit the program at any time.

## Getting your authorization token
You may use tools like [Discord-Token-Dumper by Sorrow446](https://github.com/Sorrow446/Discord-Token-Dumper) 
or do it manually by following instructions in the next two subsections:

### Desktop App
1. Add `"DANGEROUS_ENABLE_DEVTOOLS_ONLY_ENABLE_IF_YOU_KNOW_WHAT_YOURE_DOING": true` to your Discord `settings.json`.
2. Proceed by hitting `Ctrl+Shift+i` to open up the Developer Console.
3. Navigate to the `Application` tab.
4. Filter by `token`.
5. Copy the `value` corresponding to the `key` of `token`.
6. This is your token, write it to an environment variable or supply it as an argument.

### Browser App
1. Pull up the Developer Console by hitting `Ctrl+Shift+i` or `F12`.
2. On Chrome navigate to the `Application` tab, on Firefox navigate to the `Storage` tab.
3. Go to `Local Storage`.
4. Filter by `token`.
5. Copy the `value` of `token`.
6. You have your token.

## Compiling from source
```bash
git clone https://github.com/phoreverpheebs/how-active
cd how-active
cargo build
```

## Example
![example](https://user-images.githubusercontent.com/96285600/215324004-5698b81b-b667-4382-9421-ff525178b5cf.png)

# Disclaimer
The Discord TOS does say not to automate your account; whether what this program does
counts as automating or not, I will not be liable for any banned accounts.
