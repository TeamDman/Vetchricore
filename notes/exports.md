// TODO: export command to export json - purpose is to let the more tech-savvy users prepare a profile for friends that contains info 
// TODO: import command to import the exported blob


Let's say Bob wants to watch some videos with Janet.
Bob and Janet have an out-of-band communication channel that supports file uploads (e.g., Discord)

```
whoami
> Janet
vetchricore profile show
> Janet
vetchricore profile create Bob
vetchricore friend add-from-profile Janet
> Bob has added Janet as a friend.
# Janet prepares bob's profile with other information like record key or whatever
vetchricore profile export Bob bob.json
> The Bob profile has been written to bob.json, this file contains sensitive information!
```

then Janet gives that json file to Bob over Discord

```
whoami
> Bob
vetchricore profile import Bob bob.json
vetchricore friend list
> Janet
```