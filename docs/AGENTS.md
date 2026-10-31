# AGENTS.md — Public website contract

This directory is the static source for garnet-lang.org. Read the root
`AGENTS.md` before changes. Deployment is separate from local implementation
and browser verification.

## Front door

`index.html`, `front-door.css`, and `front-door.js` own the concise Experience
view, six demonstrations, platform installation controls, and shared header.
`#home`, `#use-cases`, `#install`, and `#reference` are supported deep links.
Experience is the default. Legacy #why/#code/#compare/#caps/#studio links
open their matching sections in Reference. Reference retains the complete original homepage
in `reference.html`, with its live playground, reading content, video, and
links. It is loaded once, by idle preload or on demand, into the shared shell; switching modes
must not reset its scroll position or interactive state. The shared header
replaces its original header when embedded. Keep direct Reference navigation
usable without the shell.

Experience copy is concise and declarative. Keep Live/Illustrated labels and
compact scope summaries visible; put detailed coverage, limitations, and
source links in the scenario disclosure. Keep installation requirements in
Installation details, and preserve the full technical Reference. Do not
repeat toolchain notes, implementation caveats, or explanatory disclaimers
in the hero, benefit copy, result captions, and footer.

The mode transition fades through black (220 ms in, 300 ms out). Reduced
motion and Pause motion must switch directly. Navigation must drain the
latest requested route after rapid changes, clear the veil, restore focus,
support browser Back/Forward, and preserve each view's scroll position.
Mobile navigation is a disclosure with Escape-to-close behavior. Reference
loading never holds the shell under the veil: show its loading state, require
the document marker before accepting it, and allow failed loads to retry.
Embedded links to Why, Docs, Status and the index return into the common shell
without a reload. Other outgoing pages leave the shell; section anchors stay
within Reference.
Guard access to the frame if its document becomes inaccessible.

The visitor sees one shared header, never a presentation selector or mode toolbar.
The concise homepage is the entrypoint. Activating Garnet returns to #home and
arms a gold halo; a second activation opens the original complete homepage through
black. This works with pointer, sequential touch taps, and keyboard Enter. A slow
second activation is supported; it does not depend on double-click timing. Escape
or another navigation disarms it. A rapid double-click removes its interim route
from history. Home and original hero markup share identity.css: typography, image
position, gold line, buttons and spacing must remain geometrically matched.

The Garnet wordmark and two-activation menu links carry a red diamond before
activation. The first activation returns to the intended homepage/chapter,
arms gold text or the wordmark's framed 3.6-second pulse, and exposes a concise
banner describing the next activation. Letter colours remain unchanged. The
second activation opens the deeper destination through black. Escape or other
navigation clears the armed state, glow and banner. Reduced motion retains a
static armed halo; the arrival state has no glow. Slow clicks, taps and keyboard
Enter remain equivalent. Do not reintroduce particles or an always-on pulse.

The evidence banner has four equal desktop stages: Declare, Inspect, Review,
Verify. Review describes the human decision, not an automatic acceptance proof.
At intermediate widths it forms two equal columns; narrow layouts stack the
stages. Keep numbering, baselines, spacing and terse captions consistent.

Why, Docs and Status first open their matching original homepage chapters through
black. A second activation opens their dedicated pages in the same shell. The
Why and Status start at their chapter content; Docs starts at #docs-catalogue
with a 16 px inset so its reading cards meet the header. Shell routes use the
History API and explicit routing, avoiding delayed hash replacements that can
collapse the visitor’s previous reading position.
Suppress the selected chapter’s reveal transform before measuring its anchor; the
shared black fade owns that transition and must not land on a moving target.
Explicit menu activation resets that chapter to its intended anchor; browser
history still restores independent reading positions. Chapter-to-chapter menu
navigation also fades through black. Examples opens the visual lab;
its second activation opens the original playground section. The first activation
arms gold text on those links. The Garnet wordmark always provides a home return.

Playground in the header opens #page-playground with one activation in the shared
detail frame, through the same black fade. Moving between detail pages and back
to home also uses that fade, with direct switching for reduced/Pause motion.
The opened playground stays mounted when another detail page is selected, so
returning preserves its source edits, runtime, theme and result. Hidden cached
frames must never intercept input or acquire a duplicate active-frame ID.
Standalone playground URLs and source-fragment handoffs remain native links.
The embedded playground's own home icon returns through the shared shell.
Keep focus on an armed header link so repeated keyboard Enter works without
refocusing it. No gesture is pointer-only. Modified clicks retain native links, Back/Forward and
independent scroll positions remain usable, and original frame state is preserved.

site-shell.js marks loaded pages and prepares outgoing embedded links. The shell
rejects stale responses, exposes failed-load retry, and leaves the common header
available. Original content stays mounted after its lazy preload. A different dedicated
document uses a fresh detail frame; replacing an already navigated frame can
collapse parent reading-history entries in Chromium. Cancel superseded load
listeners/timers and preserve dedicated-page scroll positions in the shell. Standalone pages
have a visible Garnet return. The hero See the boundary action explicitly starts a
real undeclared-filesystem check and its visual trace; merely loading a page does
not run the demonstration. The original hero action enters the same live journey after the fade completes,
without reloading or losing frame state. No simulated file write is presented
as execution.

`evidence-scenes.js` and `evidence-scenes.css` own the six illustrative traces.
Checker scenes must follow actual returned verdicts; they never execute host
operations. Put demonstration actions above the code so the cause and result
remain easy to discover. Reduced motion and Pause motion retain static outcomes
without pulse, packet or replay animation. At narrow widths, show the visual
and its controls before the code; use a compact horizontal example strip.
Move the same controls, preserving handlers and checked state, when crossing
the responsive breakpoint. Keep illustrative actor and memory
traces labeled and separate from runtime proof.

The first two demonstration tabs call the committed browser WASM checker
and capability-diff API. `acceptance-demo.js` owns the guided filesystem check,
the declaration diff, local Accept/Reject choice, and separately recorded CLI
journey. Checks never execute the filesystem call. The third tab is Recorded
CLI, and the remaining three are Illustrated. Browser review choices are local
UI state, not signed acceptance attestations. A passing check is not a general
safety guarantee. Do not turn
an illustration into execution evidence or broaden capability coverage in
public copy. Failed engine initialization must show an error without a check
verdict and allow an explicit retry. Retry a failed dynamic import with a
fresh same-origin module URL because browsers retain failed import promises;
successful loads use the committed module URL.

The guided check must invalidate its verdict and review choice after source
declaration changes, reset, or a stale asynchronous response. Moving to another
scenario must cancel rendering of an earlier check/record response. The editor
handoff uses the existing v1 playground fragment and never executes automatically.
`demos/acceptance/record.json` contains seven actual current-source CLI commands,
their exact exit codes, source hashes and an unsigned seal/v2. `replay.py` checks
missing/repaired declarations, the expansion exit-1 signal, byte-identical seals,
positive source verification and changed-source refusal in a temporary directory.
Regenerate it only by executing the script against a freshly built, identified
CLI. Do not present the capture as a browser-generated seal or the published
v0.8.2 behavior. Failed record loads must offer retry, without fabricated output.

`scope.html` owns the compact published/current-source/roadmap distinction.
Snapshot claims name the release, source revision and date. Capability holes,
the deferred mode bridge, memory allocator roadmap, actor-runtime integration,
converter subsets, stub registry and unmeasured production performance remain
visible in the appropriate detail. The website improves the journey; it does
not change those language/runtime contracts to satisfy marketing copy.

## Closing Inscription

The final language statement is the selected Inscription direction. Keep its
live-text uppercase two-line headline, three equal pillars with identical type
size and baseline, red periods and vertical strokes, gold baseline/diamond,
subtle decorative facets, and centered foundations/links. The geometry is
aria-hidden and cannot intercept input. Keep the existing foundations disclosure
and Why/Status links functional. Narrow layouts retain the three equal pillars
and allow the headline to wrap naturally without clipping or horizontal overflow.
The closing headline is scaled to 120 px maximum, with equally sized pillars
at 58 px maximum; preserve responsive reduction and the clear breathing room
before the final call to action. Do not replace this section with the generated concept bitmap; text must remain
selectable, zoomable, and accessible. Preserve both top heroes and the final CTA.

## Public model research

`models.html` is a public research digest of the supplied 2026-10-04 internal
Models Inside Garnet compilation, paired with Omarchy in the documentation grid.
It distinguishes authorship, advisory triage, and proposed governed model calls
from deterministic checker verdicts. Laya and Jev are research candidates, not
integrations or measured Garnet performance. The supplied compilation records
recommendations, not approvals, experiments run, or shipped model support.
Keep non-public material out of the digest; the source compilation is not
published. The models
card opens #page-models in the common shell, with a standalone Garnet return.

## Playground visual preferences

`playground/themes.js` and `playground/themes.css` provide eight adapted
workbench palettes for the Experience demonstrations and standalone playground.
Only those workspaces change; the hero and installation retain Garnet identity.
Preferences use guarded localStorage and same-origin storage events so open
playgrounds agree. Invalid preferences fall back to Garnet. Storage failure must
not disable switching. The picker and Next theme button work with keyboards;
T cycles only in a visible workspace, outside editable fields and controls,
without modifiers, repetition, or composition. Theme changes must preserve
source edits, selected examples, runtime state, and factual verdicts. This is
visual inspiration from Omarchy, not an installed OS plugin or endorsement.

Verify every palette in both workspaces across Chromium, Firefox, and WebKit,
including contrast, narrow widths, typing, preference persistence, blocked
storage, cross-tab synchronization, real WASM results, and offline reload.
Existing SHA-bound W-PLAY proofs remain historical when their HTML inputs
change; capture fresh candidate-bound browser proof at integration. Never
reseal old evidence to make this visual change look accepted.

## Installation and release truth

The release selector is pinned to the published v0.8.2 distribution. Prebuilt
CLI binaries require no Rust compiler; source builds require Rust 1.95+ and
native build prerequisites. Suggest an operating system while preserving all
three platform overrides. Browser OS detection is not processor detection:
manual processor choice is required before enabling package downloads.
macOS/Linux use the shell installer: release mode by default, source mode on request. Windows ARM uses x64
emulation. Linux release packages require glibc 2.39+. Keep SHA-256 integrity
verification distinct from separate GPG authentication. Studio is an optional
source-checkout workbench; do not offer an unpublished signed app package.
Installation controls copy commands or link assets; they never run installers.

## Assets and offline behavior

Preserve the gemstone identity and public truth markers. The service worker
must version and cache the HTML, stylesheet, application script, and WASM
needed by both modes. Video/audio continue to bypass cache handling; partial responses and newly
fetched ranged responses must not be cached. Changed shell scripts and styles use a matching revision query in their HTML
and offline cache inventory so an older cache-first service worker cannot mix
new markup with obsolete navigation code. Test a warmed-cache upgrade as well
as a cold install. Do not silently substitute stale proof or release data.

## Verification

After front-door changes, run JS syntax checks, positioning/MSRV gates, the
strict web PWA smoke check, and Playwright journeys covering both modes,
reload/deep links, Back/Forward, rapid switching, reduced motion, mobile
disclosure, six demonstrations, OS/source/package overrides, clipboard,
offline reload, and console/network health. Keep QA outputs outside tracked
source unless deliberately accepted as durable evidence. Compare actual
desktop and mobile screenshots against the approved visual references.
Building or passing static gates alone is not browser or visual acceptance.

Run the root documentation-contract ladder when changing this file.

## Authority spectrum

`authority-spectrum.js` and `authority-spectrum.css` own the local SVG picture
of eight authority kinds and a wildcard. A compact view closes the front-door
workbench; the original `#caps` chapter adds explicit examples and the complete
model. Keep the top heroes and closing Inscription unchanged.

Bands follow exact returned `garnet.wasm.check/1` and
`garnet.wasm.diff-caps/1` data. A named-call coverage diagnostic can mark a
missing kind red; a diff can mark a newly gained declaration red. Gold means
declared authority, not safety, execution, a signer or a general acceptance
proof. The status distinguishes missing and newly gained; SVG labels and
accessible text carry the meaning without colour. Ignore unknown capability
names. Invalid results, engine errors, changed inputs and inactive scenarios
clear the visual verdict; never substitute recorded output as live.

The spectrum uses the existing committed WASM bytes and local decorative gem
WebP. Reference examples check only after explicit activation, do not run host
operations, reject stale results, and offer retry after an unavailable module.
Their own chosen example survives mode round trips. Motion-off and reduced
motion remove ray pulses; narrow layouts fan below the gem with every label
inside the viewport. Match changed revision URLs and offline cache assets.

The three-character reference example is exactly `@caps()` to `@caps(net)`;
it is not a generic diff-size metric. Agent writes, checker checks, person
decides is an explanation of roles, not a claim that model-review governance
or cryptographic human acceptance is implemented by the browser. Detailed
workbench boundaries also live in `status.html#workbench-scope`.
