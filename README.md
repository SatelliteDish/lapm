# Layered Access Password manager (LAPm)
LAPm is a local, KeePass based password manager that allows you to have multiple "layers" of password protection. Each layer will be able to have it's own settings, such as if/when authentication times out.

The most common usecase for this would be a two-tiered system, with one layer for lower risk credentials and one for high risk. The lower risk layer can have a very long authentication window, or even an unlimited one. This could be for things like personal social media accounts, forum accounts, etc. Then you could have a separate layer for high risk credentials, like online banking, email, business accounts, etc., and this might require re-authentication after 30 minutes of inactivity.

## Why?
This is born out of personal need. I use KeePassXC as my daily password manager, but I find it cumbersome to have to type in my very long password for it every time I want to sign into any account.
