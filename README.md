# Websummary

This is the central repository to the websummaries we ship with our products.
It has functionality to store new components, styles
and minimize a final html from an initial 10x template.

Since websummaries may be sent via email, we try and keep the final minimized
size below 10 megabytes.

## Getting Started

These instructions will get you a copy of the project up and running on
your local machine for development and testing purposes.
See deployment for notes on how to deploy the project on a live system.

### Prerequisites

- `make`
- `yarn`
- `python`

### Installing

Clone the repository and run `make`:

```sh
git clone https://github.com/10XDev/websummary.git
cd websummary
make
```

### Running the example

The example directory contains an example JSON bag and template.
This is called in `summarize.py` for demonstration.

```sh
python summarize.py > example/example.html

# If on Linux
xdg-open example/example.html

# If on Mac
open -a 'Google Chrome' example/example.html
```

### Making Changes (Frontend)

In order to see any component changes you will have to run
`yarn build` in the `/src` directory.

For a dev build, use `yarn build-dev`.

After that is done, you can rerun the `summarize.py` script
to generate the `example.html` template.

### Extracting and testing a web summary

Let's say you want to work on the web summary here:
https://eldorado.txgmesh.net/jenkins/testresults/cellranger-master-test-pipelines_multi_sc3pv3_multiplexed_aggr/latest/2samples_3pgex_cmo_raji_jurkat_50_50/outs/per_sample_outs/raji/web_summary.html

You can extract the data into the `example` directory like so:
```sh
./extract-web-summary.sh https://eldorado.txgmesh.net/jenkins/testresults/cellranger-master-test-pipelines_multi_sc3pv3_multiplexed_aggr/latest/2samples_3pgex_cmo_raji_jurkat_50_50/outs/per_sample_outs/raji/web_summary.html > example/data.json
```

You can also manually copy/paste the html body into summary.html (TODO: automate that from the extract script?).

Then:
```sh
python summarize.py > example/example.html
```

and open `example/example.html` in a web browser.

You can then make changes (see "Making Changes (Frontend) above) and observe how the data is affected:
```sh
(cd src && yarn build) && python summarize.py > example/example.html
```


## General Usage

Call `summarize.generate_html_summary()` with a json-serializable object
describing the data (`data.json`) and the path to an HTML summary that
provides customized format to your websummary.
Also, `summarize.generate_html_summary()`
uses a general 10x template (`src/template.html`) that inlines everything
in the right place.

```python
    json_file = "example/data.json"
    contents_file = "example/summary.html"
    with open(find_local_path(json_file), 'r') as infile:
        data = json.load(infile)
    with open(find_local_path(contents_file), 'r') as infile:
        contents = infile.read()
    generate_html_summary(data, contents, os.path.join(
        os.path.dirname(__file__), 'example'), outfile)
```

### Description of Files

- `components.js` - React components that gets compiled and minified
- `template.html` - template where the compiled `components.js`,
  vendored dependencies, data, and (websummary) javascript will get inlined
- `summarize.py` - simple script that inlines all the javascript into the template

### Running the tests

If you want to check a basic Cell Ranger test before pushing a commit:

```sh
python summarize.py cr > example/my_cr_test.html
```

## Browser Compatibility

At [last evaluation](https://github.com/10XDev/websummary/pull/78), the code in this repo
supported web summaries rendered on browsers at least as new as the following:

**Browser**|**Version**|**Date**
:-----:|:-----:|:-----:
Safari|9.1|1-Sep-16
Chrome|45|9-Jul-13
Edge|15|29-May-15
Firefox|40|11-Aug-15

Current webpack configuration specifies compatibility with

**Browser**|**Version**
:---------:|:--------:
Chrome|79
Edge|94
Firefox|78
Opera|80
Safari|13.1

It does *not* work on Internet Explorer.
