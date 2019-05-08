# Password Manager

List, add, edit, and delete entries stored in a JSON file. It doesn't do much
and you definitely have better options for password management. However, I like
it and use it every day.

I have an alias for it and keep the file in Keybase.

    alias pm='pm /keybase/private/mqsoh/passwords.json'

If you want to use it, there's a Docker image for it. Add your first entry like
this.

    docker run -it --rm -v $(pwd):/workdir -w /workdir mqsoh/pm mypasswords.json add

If you want to build locally, it only requires Docker.

    git clone https://github.com/mqsoh/pm.git
    cd pm
    make shell
    # wait a long time
    cargo test

But, you probably don't want to? It's really a pet project...that I use every
day and like a lot.
