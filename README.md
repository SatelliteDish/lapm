# Layered Access Password manager (LAPm)
LAPm is a local, KeePass based password manager that allows you to have multiple "layers" of password protection.

## Why?
The problem with traditional password managers is simple, unlocking is an all or nothing ordeal: all your passwords are locked or unlocked. The problem is that there are some passwords that you'll need frequently, some will have dire consequences if leaked, and this creates a tension. For the passwords of critical importance (ie. AWS Root Account, online banking) you want a very secure password, quick timeouts, and settings that enhance security at the cost of the convenience. There are other credentials that you'll want frequent access to, and are much lower risk, such as a personal social media account. With current solutions you either have to compromise the security of your most critical credentials for the sake of convenience for the others, or you need to deal with a lack of convenience for the extra security the important credentials deserve.

Each layer is its own vault, meaning it has its own keys and can be configured entirely differently. With this, you can keep your most important credentials in a vault with a quick lockout and a highly secure password, while keeping your less important ones in a vault with no timeout and a short password. When credentials are searched it looks across all open vaults, this will work with KeePassXC browser extensions when complete.

## Installation
**Don't use this for real passwords yet**. It's far from ready, and any data saved might be completely lost or corrupted.
Right now the only method of interacting is the CLI tool. To run:
1. Clone the entire repo `git clone https://github.com/SatelliteDish/lapm`
2. Run the daemon with no arguments
3. Now the CLI should function

This should work on all platforms, but this has not been tested.

## Plans
I have lots of future plans for this project. I may not get to all these, but while work is being done these areas will be the focus. These are in no particular order
- Integrate with KeePassXC browser extensions by creating a proxy
- Upload to crates.io
- Create a GUI
- Add more layer configuration
- Add unit tests
