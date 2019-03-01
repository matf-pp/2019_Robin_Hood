# Robin Hud

Robin Hud je 2d igra iz top-down perspektive. Cilj igre je da, u ulozi Robina Huda, opljačkate zamak i pobegnete pre nego što Vas stražari primete i uhvate. Stražari patroliraju zamkom u potrazi za uljezom. Ako se igrač nađe u vidnom polju stražara, on će početi da ga juri. Ukoliko stražar uspe da ga uhvati, igra je gotova.

## Jezik/tehnologije/biblioteke

* Rust (https://rust-lang.org)
* ggez (https://github.com/ggez/ggez) - biblioteka za 2d igre za rust (događaji, grafika, itd.)
* nalgebra (https://github.com/rustsim/nalgebra) - biblioteka za linearnu algebru (vektori, tačke, matrice, itd.)
* ggez-goodies (https://github.com/ggez/ggez-goodies) - implementacija nekih korisnih dodataka na ggez

## Prevođenje 

Za prevođenje je potreban `rustc` kompilator, `rustup` alat za upravljanje verzijama rust-a (Robin Hud radi na `stable` verziji), i `cargo` za upravljanje paketima i bibliotekama.

Iz glavnog direktorijuma pokrenuti sledeću komandu.

```
cargo build --release
```
Ova komanda prevodi program i smešta ga u direktorijum `target/release/`.


## Pokretanje 

```
./target/release/robin_hood
```

## Autori
* Marijana Urošević, 186/2016, mv16186@alas.matf.bg.ac.rs
* Luka Hadži-Đokić, 269/2016, mr16269@alas.matf.bg.ac.rs
