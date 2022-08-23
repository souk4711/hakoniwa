# Apps - Xorg


## xinput

```sh
$ hakoniwa run --policy-file ./policy.toml -- /usr/bin/xinput
⎡ Virtual core pointer                          id=2    [master pointer  (3)]
⎜   ↳ Virtual core XTEST pointer                id=4    [slave  pointer  (2)]
⎜   ↳ ELAN0907:00 04F3:3183 Mouse               id=11   [slave  pointer  (2)]
⎜   ↳ ELAN0907:00 04F3:3183 Touchpad            id=12   [slave  pointer  (2)]
⎣ Virtual core keyboard                         id=3    [master keyboard (2)]
    ↳ Virtual core XTEST keyboard               id=5    [slave  keyboard (3)]
    ↳ Video Bus                                 id=6    [slave  keyboard (3)]
    ↳ Power Button                              id=7    [slave  keyboard (3)]
    ↳ Sleep Button                              id=8    [slave  keyboard (3)]
    ↳ Front Camera: Front Camera                id=9    [slave  keyboard (3)]
    ↳ Front Camera: Front IR Camera             id=10   [slave  keyboard (3)]
    ↳ Intel HID events                          id=13   [slave  keyboard (3)]
    ↳ Intel HID 5 button array                  id=14   [slave  keyboard (3)]
    ↳ AT Translated Set 2 keyboard              id=15   [slave  keyboard (3)]
```


## xterm

```sh
$ hakoniwa run --policy-file ./policy.toml -- /usr/bin/xterm
```
