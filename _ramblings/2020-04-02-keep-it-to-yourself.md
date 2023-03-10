---
layout: rambling
draft: true
---

NOTE: modhex <root PW>  
NOTE: ykman otp static 2  
NOTE: ykpersonalize -u -1 -o -fast-trig  

Keeping yourself secure in the modern age of digital computers and the internet
is a daunting task. While I like to think I'm somewhat sophisticated in this
regard, surely there are flaws in my approach, begging to be exploited and
abused. This is somewhat unavoidable, so as a general rule of thumb, **don't be
malicious**. The following is my current best practices for things that involve
information security.

---
## Table of Contents

**Section 0: Esoterica**  
A prelude on what constitutes a secret information, passwords, keys, and
notions of information security. _Feel free to skip this section._ The
definitions in this section should follow from common understanding.

**Section 1: Infrastructure**  
First we create our master password(s) and PINs. Then we build up a GPG key
structure, with some redundancy for fault tolerance. Finally we use our GPG
keys to secure secret databases of other passwords, keys, and any other data
we may want to.
1. [The Master Password(s)](#the-master-passwords-and-pins)
2. [Key Management](#key-management)
    - [GPG Keys](#gpg-keys)
        - [Smart Cards / Security Tokens](#smart-cards--security-tokens)
        - [Subkeys](#gpg-keys)
    - [SSH Keys](#ssh-secure-shell-keys)
3. [Password Management](#slave-password-management)
    - [`pass`](#pass)
    - [One-time Passwords](#one-time-passwords)
4. [Rotation](#rotation)

**Section 2: Personal Information**  
Next, we look at best practices for common kinds of personal information.
1. [General Backups](#backups)
2. [Personal Private Information](#personal-private-information)
3. [Secure Online Communication](#secure-online-communication)

---

## Esoterica

Much like how a homeowner needn't know how the pins inside their lock fall into
place, users of computer cryptosystems should be free to ignore the details of
the algorithms. Still, both must know how to keep their keys safe, and what the
limitations are. If you lock your front door, but leave the back unlocked,
you're bound to get robbed. _Trust me._ With that in mind, here's how I start
thinking about this stuff:

1. _Definition: **Security** ensures that things are as they are meant to be._

    One thing to understand about security is that it's fundamentally
    impossible, but may be approximated. You cannot build a perfect lock, seal
    a perfect record, or save anything forever. As much as entropy gives us a
    measure with which to base our security on, it also ensures it cannot last.

    It's hard to put a finger on _why_ security matters, it just does. It seems
    almost silly to try and come up with reasons, since they are all so
    visceral.  People don't want to be hurt, stolen from, cheated, or lied to.
    People want to trust the ground they stand on and live safely each and
    every day.

2. _Definition: A **secret** is some information which is unknown to others._

   We are all intimately familiar with the concept of secrets, mostly because
   they arise so naturally in our lives. If you take some food from the fridge
   without anybody noticing, it's a secret. If you plan to take the knight on
   c3 next move, it's also a secret. Some people are better at holding secrets
   than others, while conversely, some people infer the hand of a professional
   poker player by factoring their odds with the situation.

   In all cases, the secret causes an informational asymmetry, which is used in
   an number of ways. If you trust that your allies all know a secret that your
   enemies all do not know, then it 

3. _Definition: A **key** contains secret information which is used to protect
   other secrets from intrusion._

    Perhaps the most obvious example of a key is of course a door key. The
    secret information is the _bitting_ of the key, which describes how the key
    is cut precisely enough to replicate the key if needed. This is often done
    simply be taking an imprint of the original key itself. While the physical
    key is what grants you access to your house, it's the secret information
    _about the key_ that provides security.

    When we talk about software keys, it's the same thing just without the
    physical layer as directly involved. There's no lock to drill through and
    imprints can be taken by a quick copy from memory. A software key is just
    the secret information itself, which actually makes it a more fundamental
    concept than the door lock's key. There's simply no way to build a lock
    without trusting some secret information about the lock.

    If you lose your key once, there's not getting it back.


    a. _Definition: A **password** is a key which is expected to be personally
       remembered rather than stored somewhere else._

4. _Definition: An **identity** is given to all which is to be treated the
   same._

   - a door with two identical keys
   - a key that unlocks two doors
   - the holder of a private key, with a shared public key
   - ...

---

## Infrastructure

- Asymmetric Cryptography (https://gnupg.org)
- Password managers (https://www.passwordstore.org)
- U2F & https://www.yubico.com

### The Master Passwords and PINs

TODO

1. Machine Passwords
   Less secure and more numerous
2. Key passwords
   Highly secure

### Key Management

Key management through PGP is the main security concern, from which we can
build the rest somewhat easily. Luckily SmartCards can be purchased easily
these days, and make some of this much more trustworthy than it may have been
in the past. However, a single hardware device should never be trusted with all
of your secrets, since they can and will eventually fail.

#### GPG Keys

TODO:
https://www.andreagrandi.it/2017/09/30/configuring-offline-gnupg-masterkey-subkeys-on-yubikey/

At the heart of my setup lies PGP (Pretty Good Privacy), a suite of encryption
technology with well supported CLI and hardware applications. PGP allows
creating key-pairs with sub-keys (more on this later). GPG (the GNU
implementation of PGP) is used to encrypt data and manage authentication. We'll
create a new key-pair with 4096 bits. Entropy will be generated by doing the
most unusual of things to my computer in the process, you know, for security.

The primary private GPG key itself must be stored safely somewhere, like on a
keychain, and [rotated from time to
time](https://sungo.wtf/2016/11/23/gpg-strong-keys-rotation-and-keybase.html).

{ Primary, Alt } > { Subkey1, Subkey 2 }

NOTE: Rotating keys requires syncing gpg data, which requires all machines to
receive a copy of the local data. This is cumbersome, but seemingly needed.

##### Keyfiles

TODO

##### Smart Cards / Security Tokens

I own both a YubiKey 5C and 5C Nano, each with a distinct physical use case.
The traditional 5C fits onto my keychain (like with my house keys), and I can
use it to plug in and out of computers to authenticate. Meanwhile, the 5C Nano
is small enough to stay in a device, making it good for removable secure
enclave for your private keys. If you host a personal server, or simply don't
want to think about plugging in your keys then this may be your primary key. I
personally feel like the primary key shouldn't be left in the device, but
rather carried on your person. So, I keep the YubiKey 5C around as my primary
key, and leave my 5C Nano in a safe place, as a backup. They are each distinct
keys, so care must be taken to ensure data allows either key as needed.

- TODO: YubiKey setup process

If using a leave in Yubikey, disable OTP since it triggers accidentally.

##### Subkeys

In addition to SmartCard keys, conventional GPG keys can be generated and used.

- TODO: How to create these, and when to use them?
- TODO: Sub-key structure.

#### SSH (<u>S</u>ecure <u>Sh</u>ell) Keys

SSH keys should named, with the following format: `nixpulvis@<hostname>`. Each
key will be saved by name in a `pass` directory called `SSH`. This way we can
control access to individual hosts.

- TODO: Implement this.
- TODO: Note it's stored in `pass`, more on below.

Resident "discoverable" keys can be stored on a security key which supports the
FIDO standard.

**Generate an SSH key as a resident FIDO key:**
The `-t ${KEY_TYPE}-sk` tells `ssh-keygen` to use the smart card to generate
the key.
```sh
NAME=mykey
KEY_TYPE=ed25519
ssh-keygen \
   -f ~/.ssh/id_${KEY_TYPE}_sk_rk_${NAME} \
   -t ${KEY_TYPE}-sk \
   -O resident \
   -O application=ssh:${NAME} \
   -O verify-required
```

**List FIDO keys:**
```sh
ykman fido credentials list
```
The "RP ID" should be `ssh:...` as you set above. You can ignore the user and
display names.

**Recover resident SSH keys from FIDO SK:**
```sh
cd ~/.ssh
ssh-keygen -K
```


### Password Management

Once we have our key management taken care of and our master password(s) /
PIN(s) are memorized, we can rely on `pass` for managing various other
passwords and other secret data. Both our primary GPG key, and a backup will be
used for this password store. These passwords are generally generated, and
therefor not memorized.  **Some kinds of accounts may be best to memorize
still**, just make sure they aren't the same password as the master passwords
from our key management.

- TODO: Backup (stored in clear in a safe?) GPG key

`pass` stores it's data in a plain ol' folder, making it work perfectly with
`git`. In fact `pass git ...` can be used to perform arbitrary commands from
anywhere on the system. We currently store user ids in the folder name, so the
repo **must** be private. I'm currently only storing this repo on devices and
syncing with either SSH, or a USB drive.

- TODO: How to use SSH to `git fetch` from another device.
- TODO: How to use a USB drive as a git remote.

`pass` shows each record like a file within the directory tree
(`~/.password-store` by default).

```
accounts/<url>/<username> -> <password>

├── accounts
│   ├── accounts.adafruit.com
│   │   └── nixpulvis
│   ├── aur.archlinux.org
│   │   └── nathan@nixpulvis.com
│   ├── bbs.archlinux.org
│   │   └── nixpulvis
│   ├── cloud.digitalocean.com
│   │   ├── sever1
│   │   │   ├── nixpulvis
│   │   │   └── root
│   │   └── nathan@nixpulvis.com

```

```sh
# Copy Adafruit password into your clipboard.
pass -c accounts/accounts.adafruit.com/nixpulvis
```

#### Storage

The `.password-store` data will live on each machine, an encrypted USB drive
and a private repo hosted somewhere good and encrypted.

**Remember that the contents of `pass` are in an unprotected `git`
repository**.  So, while the data under each entry is protected with our keys,
someone with access to the repo can, for example, run `git log` on the changes
made to the password store. Bee careful to avoid manual commit messages that
leak information, or other non-standard `pass` practices. It's best to simply
avoid storing the password store anywhere in the clear.

- TODO: Master (in the head) passwords, re: smartcard backup

---

### Backups

I'll be using [`borg`](https://www.borgbackup.org/) to backup my machines, this
data is encrypted using a key and password that should live in `pass`. There
are a few types of Borg backups to support:

```
backup/<device> -> <password>
backup/<device>/key -> <borg_key>

├── backup
│   ├── wd2T
│   │   └── key
│   └── wd2T
```

**Blue-ray Disk Drive Backup**
- `borg` key and `pass` password stored on disk
- `keyfile*`
- Long term
- Long write time
- Manual disk drive interaction

**Local HDD/SSD Backup**
- `borg` key and `pass` password stored on device, if trusted and stored safely
- either `repokey*` or `keyfile*`
- Short term
- Fast
- Manual HDD/SSD interaction

**Remote Backup (Digital Ocean?)**
- `borg` password stored in `pass`, no key stored at all
- `repokey*` stored in `pass`
- Long term but less trustworthy
- Network write time fluctuates
- Automatic

### Personal Private Information

PII (Personally Identifiable Information) like my SSN, healthcare IDs and other
various sensitive information should be included in `pass` and nowhere else...
well, at least digitally, of course the original documents should be stored in
a safe/wallet or something.

In a system of cryptographically ensured ownership semantics (QM?), it'd be
very cool to ensure these values "lived", for lack of a better word, in my
`pass` repo. I have no idea what an implementation of this would look like
though.

Other things can be saved in `pass` too, like tax, healthcare, finance, or any
other sensitive information.

```
<note>/* -> <note>

├── health
│   └── ...
└── taxes
    ├── 2017
    └── 2018
```

Other people's contact information is private. We can store both public names
with their fingerprints (or other potentially private information):

```
contacts/<name> -> <gpg_fingerprint>

├── contacts
│   ├── Nathan Lilienthal
│   ├── Bob Smith
│   │   ├── Personal
│   │   └── Work
│   ├── Jane Doe
```

- TODO: Private set of contact names
