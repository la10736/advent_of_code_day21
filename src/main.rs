#![feature(conservative_impl_trait)]

use std::io::prelude::*;

fn read_all<S: AsRef<std::path::Path>>(path: S) -> String {
    let mut content = String::new();
    let mut f = std::fs::File::open(path).unwrap();
    f.read_to_string(&mut content).unwrap();
    content
}

fn main() {
    let steps = std::env::args().nth(1).unwrap_or(String::from("2")).parse().unwrap();
    let fname = std::env::args().nth(2).unwrap_or(String::from("example"));
    let content = read_all(fname);

    let book = content.parse::<Book>().unwrap();

    let mut f = Fractal::new(&book);

    (0..steps).for_each(|s| {
        f.next();
        println!("[{}] Ones = {}", s + 1, f.ones());
    });

    println!("Ones = {}", f.ones());
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
struct Block {
    conf: Vec<bool>,
    size: usize,
}

impl Block {
    fn new(size: usize) -> Self {
        Self { conf: vec![false; (size * size)], size }
    }

    fn flip_h(&self) -> Self {
        let desc = self.desc().lines().map(
            |l| l.chars().rev().collect::<String>()
        ).collect::<Vec<_>>().join("\n");
        Self { conf: Self::desc2conf(desc.clone()), size: self.size }
    }

    fn desc2conf(desc: String) -> Vec<bool> {
        desc.chars()
            .filter(|&c| c != '\n')
            .map(|c| c == '#')
            .collect::<Vec<_>>()
    }

    fn flip_v(&self) -> Self {
        let desc = self.desc().lines().rev().collect::<Vec<_>>().join("\n");
        Self { conf: Self::desc2conf(desc.clone()), size: self.size }
    }

    fn pos(&self, r: usize, c: usize) -> usize {
        (r * self.size) + c
    }

    fn pixel(&self, r: usize, c: usize) -> bool {
        self.conf[self.pos(r, c)]
    }

    fn desc(&self) -> String {
        let c_s = self.conf
            .iter()
            .map(|&v| if v { '#' } else { '.' })
            .collect::<String>();
        (0..self.size)
            .map(|i| &c_s[(i * self.size)..((i + 1) * self.size)])
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn coords<'a>(&'a self) -> impl Iterator<Item=(usize, usize)> + 'a {
        let size = self.size;
        (0..(size * size))
            .map(move |i| (i / size, i % size))
    }

    fn rotate(&self, amount: usize) -> Self {
        let amount = amount % 4;
        if amount == 0 {
            return self.clone();
        }

        let size = self.size;
        let rule = |r: usize, c: usize| match amount {
            1 => self.pixel(size - c - 1, r),
            2 => self.pixel(size - r - 1, size - c - 1),
            3 => self.pixel(c, size - r - 1),
            _ => unreachable!()
        };

        let conf = self.coords()
            .map(|(r, c)|
                rule(r, c)
            )
            .collect::<Vec<_>>();

        Self { conf, size }
    }

    fn slice(&self, rows: std::ops::Range<usize>, cols: std::ops::Range<usize>) -> Self {
        assert_eq!(rows.len(), cols.len());
        let size = rows.len();
        Self {
            conf: rows.flat_map(|i| cols.clone().map(move |j| self.pixel(i, j))).collect(),
            size,
        }
    }

    fn split(&self, step: usize) -> Vec<Self> {
        assert_eq!(self.size % step, 0);

        let blocks = self.size / step;
        (0..blocks).flat_map(|i| (0..blocks).map(move |j| (i, j)))
            .map(
                |(i, j)| self.slice(
                    (i * step)..((i + 1) * step),
                    (j * step)..((j + 1) * step))
            )
            .collect()
    }

    fn blit(&mut self, coord: (usize, usize), block: &Self) {
        assert!(coord.0 + block.size <= self.size);
        assert!(coord.1 + block.size <= self.size);

        block.coords().for_each(
            |(r, c)| {
                let p = self.pos(r + coord.0, c + coord.1);
                self.conf[p] = block.pixel(r, c);
            }
        )
    }

    fn ones(&self) -> usize {
        self.conf.iter().filter(|&p| *p).count()
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.desc())
    }
}

impl std::str::FromStr for Block {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let desc = s.replace("/", "\n");
        Ok(Self { conf: Self::desc2conf(desc.clone()), size: desc.clone().lines().count() })
    }
}

impl<S: AsRef<str>> From<S> for Block {
    fn from(s: S) -> Self {
        s.as_ref().parse::<Block>().unwrap()
    }
}

impl AsRef<Block> for Block {
    fn as_ref(&self) -> &Self {
        self
    }
}

struct Book {
    entries: std::collections::HashMap<Block, Block>,
    classes: std::collections::HashMap<Block, Block>,
}

impl Book {
    fn resolve<B: AsRef<Block>>(&self, b: B) -> Option<&Block> {
        self.entries.get(self.classes.get(b.as_ref())?)
    }
}

trait SplitTwo {
    fn split_two(&self, token: &str) -> Option<(&str, &str)>;
}

impl<S: AsRef<str>> SplitTwo for S {
    fn split_two(&self, token: &str) -> Option<(&str, &str)> {
        let s = self.as_ref();
        let p = s.find(token)?;
        Some((&s[..p], &s[p + token.len()..]))
    }
}

impl std::str::FromStr for Book {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entries = s.lines().map(
            |l| {
                let (from, to) = l.split_two(" => ").unwrap();
                (Block::from(from), Block::from(to))
            }
        ).collect::<std::collections::HashMap<_, _>>();

        let classes = entries.iter().flat_map(
            |(f, _)|
                vec![f.flip_h(), f.flip_v(), f.clone()].into_iter().
                    flat_map(|t|
                        vec![t.rotate(0), t.rotate(1), t.rotate(2), t.rotate(3)].into_iter()
                    ).map(move |e| (e, f.clone()))
        ).collect();

        Ok(Self { entries, classes })
    }
}

struct Fractal<'a> {
    book: &'a Book,
    state: Block,
}

impl<'a> Fractal<'a> {
    fn new(book: &'a Book) -> Self {
        Self {
            state: ".#./..#/###".parse::<Block>().unwrap(),
            book,
        }
    }

    fn ones(&self) -> usize {
        self.state.ones()
    }

    fn image(&self) -> &Block {
        &self.state
    }

    fn next(&mut self) {
        let b_size = if self.state.size % 2 == 0 { 2 } else { 3 };
        let blocks = self.state.split(b_size)
            .iter().map(|b| self.book.resolve(b).unwrap()
        ).collect::<Vec<_>>();

        let blocks_for_side = (blocks.len() as f64).sqrt() as usize;
        assert_eq!(blocks_for_side * blocks_for_side, blocks.len());


        let b_size = blocks[0].size;
        let mut state = Block::new(blocks_for_side * b_size);

        for (p, &b) in blocks.iter().enumerate() {
            state.blit((p/blocks_for_side * b_size, p%blocks_for_side * b_size), b)
        }

        self.state = state;
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn b(s: &str) -> Block {
        s.parse().unwrap()
    }

    static BOOK: &'static str = "../.# => ##./#../...\n\
                                     .#./..#/### => #..#/..../..../#..#\
                                     ";

    mod image {
        use super::*;

        #[test]
        fn image() {
            let book = BOOK.parse::<Book>().unwrap();
            let f = Fractal::new(&book);

            assert_eq!(&Block::from(".#./..#/###"), f.image())
        }

        #[test]
        fn simple_step() {
            let book = BOOK.parse::<Book>().unwrap();

            let mut f = Fractal::new(&book);

            f.next();

            assert_eq!(4, f.ones())
        }

        #[test]
        fn some_steps() {
            let book = BOOK.parse::<Book>().unwrap();

            let mut f = Fractal::new(&book);

            f.next();
            f.next();

            assert_eq!(12, f.ones())
        }
    }

    mod block {
        use super::*;

        #[test]
        fn parse_from_str() {
            let block: Block = b(".#./###/#..");

            assert_eq!(".#.\n\
                        ###\n\
                        #..", &format!("{}", block))
        }

        #[test]
        fn parse_full() {
            assert_eq!(4, b("##/##").ones());
            assert_eq!(9, b("###/###/###").ones());
            assert_eq!(16, b("####/####/####/####").ones());
        }

        #[test]
        fn size() {
            assert_eq!(3, b(".../#.#/##.").size);
            assert_eq!(2, b(".#/#.").size);
            assert_eq!(4, b(".#.#/#.##/..##/##..").size);
        }

        #[test]
        fn flip_horizontal() {
            assert_eq!(b("#../##./.##"), b("..#/.##/##.").flip_h());
        }

        #[test]
        fn flip_vertical() {
            assert_eq!(b("##./.##/..#"), b("..#/.##/##.").flip_v());
        }

        #[test]
        fn rotate_one_step() {
            assert_eq!(b(".../.../..."), b(".../.../...").rotate(1));
            assert_eq!(b("..#/.../..."), b("#../.../...").rotate(1));
            assert_eq!(b(".../.../..#"), b("..#/.../...").rotate(1));
            assert_eq!(b("#../##./.##"), b("..#/.##/##.").rotate(1));
            assert_eq!(b("##../..#./###./#..#"), b("...#/.##./#.#./#.##").rotate(1));
        }

        #[test]
        fn rotate() {
            assert_eq!(b("..#/.##/##.").rotate(1).rotate(1),
                       b("..#/.##/##.").rotate(2));
            assert_eq!(b("..#/.##/##.").rotate(1).rotate(1).rotate(1),
                       b("..#/.##/##.").rotate(3));
        }

        #[test]
        fn rotate_0() {
            assert_eq!(b("..#/.##/##."),
                       b("..#/.##/##.").rotate(0));
        }

        #[test]
        fn rotate_modulo() {
            assert_eq!(b("..#/.##/##."),
                       b("..#/.##/##.").rotate(4));
            assert_eq!(b("..#/.##/##.").rotate(3),
                       b("..#/.##/##.").rotate(7));
        }

        #[test]
        fn split_by_2() {
            assert_eq!(vec![b("#./.."), b(".#/.."), b("#./.#"), b(".#/#.")],
                       b("#..#/..../#..#/.##.").split(2))
        }

        #[test]
        fn split_by_3() {
            assert_eq!(vec![b("#.#/.##/..#"), b("..#/..#/#.."),
                            b("##./..#/#.#"), b(".##/#../.#.")],
                       b("#.#..#/.##..#/..##../##..##/..##../#.#.#.").split(3))
        }

        #[test]
        fn blit() {
            let mut block = Block::new(6);

            block.blit((1, 2), &b("#.#/.##/..#"));

            assert_eq!(b("....../..#.#./...##./....#./....../......"),
                       block);
        }
    }

    mod book {
        use super::*;

        #[test]
        fn resolve_happy_path() {
            let book = BOOK.parse::<Book>().unwrap();

            assert_eq!(&b("##./#../..."), book.resolve(b("../.#")).unwrap());
            assert_eq!(&b("#..#/..../..../#..#"), book.resolve(b(".#./..#/###")).unwrap());
        }

        #[test]
        fn parse_long_line() {
            let book = "#.#/###/#.# => ##../..../..../####".parse::<Book>().unwrap();

            let result = book.resolve(b("#.#/###/#.#"));

            assert_eq!(&b("##../..../..../####"), result.unwrap());
        }

        #[test]
        fn resolve_rotated() {
            let book = BOOK.parse::<Book>().unwrap();

            let result = book.resolve(b("../#."));

            assert_eq!(&b("##./#../..."), result.unwrap());
        }

        #[test]
        fn resolve_rotated_and_flipped() {
            let book = BOOK.parse::<Book>().unwrap();

            let result = book.resolve(b("##./#.#/#.."));

            assert_eq!(&b("#..#/..../..../#..#"), result.unwrap());
        }
    }
}
