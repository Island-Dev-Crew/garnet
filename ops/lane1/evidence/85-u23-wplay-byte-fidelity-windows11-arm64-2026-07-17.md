# U-23 — W-PLAY byte-fidelity gap on a default Windows clone

Date: 2026-07-17  
Status: OPEN; Lane 0 repair #3 input; not repaired in this successor  
Guest: Microsoft Windows 11 Pro build 26200, ARM64-based UTM machine  
Probe process: AMD64 compatibility environment  
Repository: fresh `C:\garnet-main-cede` clone  
Head: `cede73c03c5d535306ed179b5882e99e4d17b050`  
Git: `core.autocrlf=true`

The four named F1 readiness suites are already RED on Windows main. The
W-PLAY proof, provenance, and raw runtime-input closure is not protected by a
byte-exact `.gitattributes` fence. The gates behaved correctly; this successor
does not change those gates, artifacts, or attributes.

The exact `git check-attr text eol -- <path>` output and SHA-256 comparison of
`git cat-file blob HEAD:<path>` against the checked-out file follow. Nine text
paths diverge. The three binary paths remain identical; only the Lane 2A
evidence screenshot is explicitly `text: unset` today.

| Path | `text` | `eol` | Object SHA-256 | Worktree SHA-256 | Equal |
|---|---|---|---|---|---|
| `apps/garnet-studio/package-lock.json` | unspecified | unspecified | `e729ee69006fd3e6f5aa6171a93b8477ec51e96f12bb476c1a923df46aa93422` | `84569af4a609760b5a8585f97d13410303dba0d07e105476f0f43762ac79de8c` | false |
| `apps/garnet-studio/package.json` | unspecified | unspecified | `89aedfa9bf42d4b11e3dc073600675f2df8dfaf46f3e4313ae99ad7a97ad6c41` | `b9156ea78137cc6477ae3f6b698513f3967cdd432bba60534273168796e08931` | false |
| `docs/icons/garnet-192.png` | unspecified | unspecified | `790a70b22ea92e6f7004eec6196c0807f612d149a2bb40115d31d44ab68927fa` | `790a70b22ea92e6f7004eec6196c0807f612d149a2bb40115d31d44ab68927fa` | true |
| `docs/playground.html` | unspecified | unspecified | `01b12abd2046ebe61de8b7d0b4fa45430981211e68d99ec32a1b3cf7ae775942` | `06710320417e06214525ea587a2718ed41a2e8f622229616708dc9e1ae7de1be` | false |
| `docs/playground/examples.json` | unspecified | unspecified | `eb1b7edb207c2a695b7d51c5b3c4acfac51bf42eb706c0da520b5586a0534d18` | `8daf2e3be7dd923dd52b3811fb0bf5ff4f75901f5cf2b9576f81ea5abaf916bf` | false |
| `docs/playground/live.js` | unspecified | unspecified | `0c9b32f144a5acbbeb36c29c1822d12ec2293d2ea29a0a10d8e5432d4a958c91` | `d67a0656ea031869ff5fde7eeb50c686725c528392132a683d453695467ab203` | false |
| `docs/playground/pkg/garnet_wasm.js` | unspecified | unspecified | `bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d` | `be24dbeff8765dc7c4e97f2604c8616527710a7ab4322aaacbd2792f2b0ac820` | false |
| `docs/playground/pkg/garnet_wasm_bg.wasm` | unspecified | unspecified | `1b54c5ddefa045279c8cae4a92b030d43b7631595639ba02ac075a1e34e55e15` | `1b54c5ddefa045279c8cae4a92b030d43b7631595639ba02ac075a1e34e55e15` | true |
| `docs/playground/pkg/provenance.json` | unspecified | unspecified | `a7d9c6e3f9667ae10b90d9f0a276ce1cf65e2596b5bf807629f1e11256e2a2ee` | `912c1a845d606445b711336fdbf6c0f3fb6b8f27809ad1b1c9032c322eeaf9f4` | false |
| `scripts/smoke_garnet_playground_browser.mjs` | unspecified | unspecified | `ddbd4ac831a8cc108514527f70ae06ea58b0e8064b51a18897d4c08348d3978d` | `f965efa14ae882ca29abe796a65e3033f14f01a1096e115a63d4cd7dcd2bbfab` | false |
| `F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json` | unspecified | unspecified | `9ed58924d7652de1318862db8cb2510bf6808393f17d92fe895c946b1fef89a0` | `dcaf84e3419d0fb141019e46380681012527d2109add193fd1a3a68c6b14a2a8` | false |
| `ops/lane2a/evidence/30-playground-browser.png` | unset | unspecified | `c9296819d2fafc75fd6da60411932a32c3da65d3ea55e3243be576e2365bd480` | `c9296819d2fafc75fd6da60411932a32c3da65d3ea55e3243be576e2365bd480` | true |

Lane 0 repair #3 must decide the byte-exact fence for this complete closure and
prove a fresh default-Windows clone. This packet does not prescribe or apply
that repair.

