# For sorting bib files (LaTeX)

The idea is: You have bib files containing items like 

```bib
@article{boers2019,
    author = {N. Boers AND B. Goswami AND A. Rheinwalt AND B. Bookhagen AND B. Hoskins AND J. Kurths},
    title = {Complex networks reveal global pattern of extreme-rainfall teleconnections},
    journal = {Nature},
    year = 2019,
    volume = {566},
    pages = {373-377},
    doi = {10.1038/s41586-018-0872-x}
}

@Book{baxter1982,
  author =    {R. J. Baxter},
  title =     {Exactly Solved Models in Statistical Mechanics},
  publisher = {Academic Press},
  address =   {London},
  year =      {1982},
}
```

and the bib item should be sorted according to the key (here boers2019 and baxter1982).
The sorting is case insensitive.

It will print the sorted files to the terminal

## Compiling

If you do not have Rust (and cargo) installed, install it [https://doc.rust-lang.org/book/ch01-01-installation.html].
Then clone this repository. Go into the folder.
Compile via:
```bash
cargo b -r
```

The executable "bib_sort" is now in the folder ./target/release

