# The Reconciliation: How a Question in Bed Became a Programming Language

> *"For my thoughts are not your thoughts, neither are your ways my ways, saith the LORD."* — Isaiah 55:8 (KJV)

---

## Saturday Morning

Showered, dried off, and laying in bed — butt ass naked — thinking about whether I wanted to take a nap or head to my mother's house for Sabbath lunch.

I had a podcast running. I don't normally do podcasts, but there was this guy having a conversation talking about how beautiful Ruby code is. Describing why Ruby is beautiful. How it reads. How the code flows like something written by a person who cares about the person reading it.

And I thought: *that reminds me of God.*

Not in a weird way. In the sense that God is organized, orderly, and beautiful. Nothing He creates is chaotic. The universe has patterns — laws — and they are elegant when you can see them.

I later learned the guy was DHH on *The Pragmatic Engineer* podcast.

But then: *everybody's talking about Rust right now. In AI, I thought.*

So I asked the question —

**what's the difference between Rust and Ruby? And can we reconcile them?**

Not "can we pick the better one." Can we put them together?

I picked up my phone and sent a paragraph — just one — into Claude's research mode. Not Pulse, because Pulse checks me when I ask things on Sabbath and I wasn't about to get the "should I answer this, it's the Sabbath" lecture again. 🤣 I snuck around my own assistant.

I didn't even expect work. I just wanted answers. Research. Since I was on the Max plan and it was expiring soon, I figured I might as well use it for something that didn't require building — just thinking.

**Claude spent 58 minutes reading 800+ sources.** By the time I got clothes on, drove to my mother's, and sat down for Sabbath lunch, it was finished and waiting for me.

That was the spark.

## Sabbath to Saturday

I read the research at lunch. And realized: oh, this is something pretty substantial.

By Sabbath evening — after food, after family, after rest — I started building. Put together the deeper research side by side across multiple surfaces and frontier models.

But I didn't trust one AI. So I built a council.

Claude first. Then ChatGPT with extended thinking. Then Grok 4.2 with multi-agent research. Then back to Claude to validate the gaps. Then Gemini Deep Research — almost forgot Gemini, but they just finished a quantum memory study so their insight on the memory architecture side was deeper than anything else.

Then I took everything back to Claude one more time and said: *"Synthesize all of it. Make sure nothing is missing."*

Six passes. Five models. One architecture.

I told every single one the same thing: *"This is going to be presented at MIT. Scrutinized by people who know more than me. Miss nothing."*

They didn't hold back.

## The Clock Was Ticking

Here's what the public won't see unless I tell them:

My utilities were cut off **twice** this month. I was on a $100 Claude Max plan, then paid another $100 to stay in — $200 total. That $200 had to cover everything: Garnet, Jimmy Wilson's AI education project at Oakwood, and whatever else needed doing.

I had about two weeks at $200/month. That was it. After that, the window closed.

My mother looks at me like I'm crazy because I keep telling her I'm building something amazing and she keeps asking: *"Where's the money?"*

She's right to ask. That's why I'm trying to get this stuff organized and visible — so the company can breathe and we can keep going.

## April 12 to April 17

Six days. That's how long it took from the moment the build started to the moment Claude Max cut off and I couldn't push anymore.

- Sunday, Apr 12: build begins after Sabbath
- Monday–Friday: built through the nights. Zero sleep. Then straight to work the next day. Repeat. Sabbath evenings became build windows — after sundown, after rest, I'd sit down and push. Opus 4.7 dropped mid-sprint. Full plan review, gap-filling, enhancement.
- Friday, Apr 17: Claude Max expires. Garnet is at v4.2 — a working, tested, cross-platform installable language system with 1,244 tests committed, 136 security tests, 22K lines of Rust source, seven research papers, and a 1,670-line specification.

That was day six.

## What Garnet Is

Garnet is a programming language with two modes in one file:

- **Managed mode** (`def`) — reads like Ruby. Fast. Expressive. Joyful to read. ARC memory, dynamic-ish types.
- **Safe mode** (`@safe fn`) — reads like Rust. Ownership, borrowing, zero-cost abstractions. Mathematically-grounded safety.

And then there's the memory system. No other language has first-class memory primitives. Garnet has four: working, episodic, semantic, and procedural. You declare them like variables:

```garnet
memory episodic SessionLog : EpisodeStore<Interaction>
memory semantic KnowledgeBase : WeightedGraph<Concept>
```

That's not a library import. That's the language itself understanding that agents need memory the way humans do.

The logo is half Rust, half Ruby, with a memory core at the center. The name Garnet comes from the gemstone — formed under pressure, compressed, hardened, and beautiful because of the force, not despite it.

## Where It Stands

As of April 17:

- **1,244 tests** across 7 crates + **136 security tests**
- Linux installer verified (.deb + .rpm in Docker)
- Windows binary verified (MSVC native)
- macOS binary compiled (pending Apple notarization credentials)
- Universal shell installer (`curl sh.garnet-lang.org | sh`) — shellchecked clean
- Ed25519 signed build manifests
- Ruby/Rust/Python/Go → Garnet converter (85 tests)
- Seven research papers, four addenda, seven empirical contributions
- Dual Apache-2.0/MIT license (same as Rust)
- Domain registered: garnet-lang.org

What remains is mechanical. Signing credentials. macOS notarization. GitHub repo creation. None of it is architectural. The language works. It installs. It builds. It runs.

## What I Don't Have

I don't have the $200 to renew Claude Max. I don't have investors. I don't have a team. I have a MacBook Air, a folder of artifacts that represent six of the most intense days of my life, and a conviction that this matters.

My mother's right: there's no money yet. But there's something real, and there's something that exists, and it didn't exist before April 11.

## Why This Matters

Not because a solo founder built a language in six days — that's a party trick if it doesn't solve a real problem.

It matters because the industry is building agentic systems on languages that weren't designed for agents. We're bolting memory onto Python with Redis. We're wiring message passing into Go by hand. We're fighting borrow checkers in Rust for orchestration code that should be simple.

Garnet says: *what if the language itself understood that agents have memory, need to communicate safely, and require authority boundaries that are visible at compile time?*

That's the bet.

## The Verse

*"Where there is no vision, the people perish."*

Proverbs 29:18. I kept seeing it throughout the build. Not because I'm trying to make this spiritual. Because that's the truth. I was laying in bed on a Saturday morning with a question, and seven days later I had a language. The vision was there before the code was. The code followed the vision — it didn't create it.

---

*Jon Isaac — Island Development Crew — Huntsville, Alabama*
*April 19, 2026*
