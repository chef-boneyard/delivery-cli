To Dos
======

- STDIN / TTY issues testing password input
- Check that a `delivery` remote exists before attempting to push

  Right now we just pass out the git error. It'd be a better user
  experience (we could suggest they run `delivery init`) and it'd be
  easier to test because we wouldn't need to mock the error message in
  our fake `git` binary
