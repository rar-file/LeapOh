# LeapOh

> *htop is staring. This is glancing.*

A calm CLI that compresses your morning into a single command. You run it when you arrive at your terminal, when you `cd` into a project, when you want to see your day. Then it's gone. No background daemon. No perpetual frame.

```
   .---.   z   good morning, Cam — Friday · 08:14
  /     \ Z
*( -   - )*z   git  main · 3 changed · 2 ahead
  \  ~  /      gh   2 review requests waiting
   '---'       wx   14° clear
   /   \       cal  standup in 26m
  /__|__\      ◐    day 7 of your streak
   |   |
               axo says: "good morning. you've got a busy one."
```

The pet's mood compresses everything. The four lines next to it spell it out.

---

## Why it earns "useful"

A clean pet means a clean morning. A concerned pet means something needs you. One glance replaces five commands — `git status`, the weather widget, github.com, your calendar, the streak counter — and you go back to work.

It's not a dashboard. Dashboards stare. This glances.

---

## Install

```sh
git clone https://github.com/rar-file/LeapOh
cd LeapOh
cargo install --path .
```

Single static binary, ~2 MB. Warm startup: ~25 ms.

---

## Usage

```sh
leapoh                          # the glance
leapoh --pose excited           # preview a mood (debug sprites)
leapoh --creature ./mine.toml   # try your own creature
leapoh --help
```

Add it to your shell's prompt or `chpwd` hook if you want it on every `cd`. Or just type `leapoh` when you sit down.

---

## Configuration

`~/.config/leapoh/config.toml` — every key is optional:

```toml
creature         = "axolotl"   # built-in name, or a path to a sprite TOML
name             = "Cam"       # how the creature addresses you
workday_start    = 9
workday_end      = 18
max_observations = 5
observers        = ["git", "github", "weather", "system", "streak"]

[location]                     # required only for weather
lat = 40.7128
lon = -74.0060
```

---

## The poses

The creature picks a pose by reading your environment. Priority is fixed — the most urgent signal wins.

| Pose         | Triggered when                                                |
|--------------|---------------------------------------------------------------|
| `sick`       | battery < 10% off AC, or disk > 95% used                      |
| `adorned`    | streak milestone (7, 14, 30, 60, 100, 200, 365…)              |
| `shivering`  | local temperature < 5° C                                      |
| `excited`    | merged something in the last hour                             |
| `concerned`  | dirty tree > 24h, low disk, or ≥ 3 review requests waiting    |
| `sleeping`   | hour < 6 or ≥ 23                                              |
| `working`    | workday hour + currently in a git repo                        |
| `idle`       | the default                                                   |

Force any pose with `--pose <name>` to preview your sprite set.

---

## Forking the creature

Poses are TOML. Anyone can fork the axolotl without recompiling.

```toml
name         = "duck"
default_pose = "idle"

[poses.idle]
art = '''
   __
  <(o )___
   ( ._> /
    `---'
'''

[poses.sleeping]
art = '''
   __    z
  <(- )_  Z
   ( ._> /
    `---'
'''
```

Drop it in `~/.config/leapoh/creatures/duck.toml`, then set `creature = "duck"` in your config — or pass `--creature ./duck.toml` for one-off use.

---

## Observers

An observer is a small unit that returns `(priority, tag, line)` plus structured signals that feed pose selection. Built-ins live in `src/observers/`:

- **git** — branch, dirty count, unpushed, recent merges
- **github** — review requests, notifications (cached 60 s)
- **weather** — open-meteo current conditions, given lat/lon
- **system** — battery, disk
- **streak** — days you've actually shown up

Observers run in parallel. The slow ones don't hold the glance.

Want a new one? Implement the `Observer` trait, register it in `observers/mod.rs::active`, give it a tag and a priority floor. That's the whole contract.

---

## Architecture in one paragraph

`main` gathers a `Context` (time, cwd, user, config), runs every enabled `Observer` in a `thread::scope`, merges their `Signals` and `Observation`s, asks `pose::select` for the right mood, loads the sprite, picks the pose's ASCII art, asks `voice::synthesize` for the bottom line, and `render` lays it out left-creature / right-text. Then the process exits. That's it.

---

## Roadmap

- `calendar` observer (gcalcli, icalbuddy, raw `.ics`)
- `pomodoro` sidecar (hearth-pomo style)
- More creatures (panda, duck, otter)
- Optional color themes
- Sprite hot-reload while you tweak ASCII

---

## Philosophy

> A clean pet means a clean morning. A concerned pet means something needs you.

LeapOh is built on the bet that *mood is a higher-bandwidth channel than text*. You can read a creature's face in 80 ms. You can't read four lines of `git status` that fast. The lines next to the creature are the *receipt* for the mood — they tell you which signal moved the needle. But the mood is what you came for.

If you ever feel like you're *looking at* LeapOh instead of *glancing at* it, that's a bug. Tell us.

---

## License

MIT.
