# --user

Set user for the container

## Use default group, and default supplementary groups

```console
$ hakoniwa run --rootfs ../hakoniwa/tests/fixtures/rootfs --userns=auto --user root -- id
uid=0(root) gid=0(root) groups=0(root),1(bin),2(daemon),3(sys),4(adm),6(disk),10(wheel),11(floppy),20(dialout),26(tape),27(video)

```

## Use specified group, no supplementary groups

```console
$ hakoniwa run --rootfs ../hakoniwa/tests/fixtures/rootfs --userns=auto --user root:root -- id
uid=0(root) gid=0(root)

```

## Use specified group, and specified supplementary groups

```console
$ hakoniwa run --rootfs ../hakoniwa/tests/fixtures/rootfs --userns=auto --user root:root:wheel,video -- id
uid=0(root) gid=0(root) groups=10(wheel),27(video)

```
