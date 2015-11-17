### pbooks - downloads selected books from vhf/free-programming-books

##### Crates to use
* iron - to download the books and serve the books if need be (concurrent)
* ncurses - for selecting the books
* toml or something for storing user selection and correct checksums
* twox-hash - for checking integrity of files and generation checksums
* zip and tar - for creating distributable packages
* clippy - good code

##### Plans
* UI of some sort to pick categories?
* Store choices in some files?
* Check if file exists before downloading - check hashes
* Maybe serve the books? - Useful for classes
