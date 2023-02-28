mod ast;
mod parse;
mod tokenize;

pub use ast::Scope;

// fn count_invocations(&[Code]) -> usize {
//     let mut out = TokenStream::new();
//     let mut iter = stream.into_iter();
//     let mut count = 0;

//     while let Some(mut tt) = iter.next() {
//         if let TokenTree::Group(group) = &tt {
//             let (_, subcount) = count_branches(group.stream());

//             count += subcount;
//         } else if tt.is("html") {
//             out.write(tt);

//             tt = match iter.next() {
//                 Some(tt) => {
//                     if tt.is('!') {
//                         count += 1;
//                     }
//                     tt
//                 }
//                 None => break,
//             }
//         }

//         out.write(tt);
//     }

//     (out, count)
// }
