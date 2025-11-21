# Contributing to OmniLED

First off, thank you for your interest in contributing to OmniLED! All contributions are welcome, whether you are fixing
bugs, improving documentation, suggesting new features, or sharing device configurations.

If you are getting started with contributing to open source, please check out
the [GitHub documentation](https://docs.github.com/en/get-started/quickstart/contributing-to-projects), to see how to
fork the repo and open pull requests.

## Development Setup

1. Install all dependencies, see the [installation instructions](docs/install.md).
2. Create a fork of the project.
3. Run `cargo make test` to build and test the project to make sure everything works.

## Coding style

- Make sure the code is formatted using `cargo fmt`.
- Try to follow the existing code style, there are no strict rules.
- `unsafe` code is *allowed*, but please make sure you explain why it is safe and why it's required.
- For documentation, please follow the existing style. That includes the 120-character line limit.

## Reporting Issues

Before opening a new issue, please check
the [previously open issues](https://github.com/llMBQll/OmniLED/issues?q=is%3Aissue) and make sure your problem hasn't
been already reported or resolved.

## Documentation Improvements

Documentation improvements are always appreciated. If you feel the documentation is lacking, sentences could be clearer,
or if you just found a typo, please open a Pull Request with your changes.

## Adding Configurations for Tested Devices

If you have managed to get OmniLED working with a device that is not yet officially supported, please consider sharing
your configuration to help others.

When submitting:

1. Ensure your configuration is tested.
2. Open a PR with the new configuration file.
   > **Example:** [Add default config for SteelSeries Apex 5](https://github.com/llMBQll/OmniLED/pull/21/)

## New features

1. Please open an issue first and describe how the feature should work. You can add images or other means of describing
   the new functionality. This gives the chance for everyone to weigh in on the design.

   Of course, you may start out with some proof of concept code, but keep in mind that your idea may require some
   redesign after discussion.
2. Open a PR with the new feature.
3. Make sure all tests pass.

> If you are unsure about the design, feel free to open an issue first and ask for feedback.  
> The same goes for when you don't know how to implement a feature, but you think it would be useful. Any contributor
> can pick it up.

## Handling issues

If an issue is not assigned to anyone, you can ask to be assigned to it, and then you can handle the work. This ensures
that two people don't work on the same thing.

## Licensing

OmniLED is licensed under the GNU General Public License v3.0 (GPLv3).
By submitting any contribution (including pull requests, code, configurations, or documentation), you agree that your
work will be licensed under the same GPLv3 license as the rest of the project.
