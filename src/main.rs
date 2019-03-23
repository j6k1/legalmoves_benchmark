extern crate usiagent;

use std::time::{Instant};
use std::collections::HashMap;

use usiagent::rule::*;
use usiagent::protocol::*;
use usiagent::event::*;
use usiagent::shogi::*;

const DEPTH:u32 = 4;
const INITIAL_SFEN:&'static str = "sfen l6nl/5+P1gk/2np1S3/p1p4Pp/3P2Sp1/1PPb2P1P/P5GS1/R8/LN4bKL w RGgsn5p 1";

fn main() {
	let position_parser = PositionParser::new();

	let (teban, banmen, mc, _, _) = match position_parser.parse(&INITIAL_SFEN.split(" ").collect::<Vec<&str>>()).unwrap() {
		position => match position {
			SystemEvent::Position(teban, p, n, m) => {
				let(banmen,mc) = match p {
					UsiInitialPosition::Startpos => {
						(BANMEN_START_POS.clone(), MochigomaCollections::Pair(HashMap::new(),HashMap::new()))
					},
					UsiInitialPosition::Sfen(ref b,MochigomaCollections::Pair(ref ms,ref mg)) => {
						(b.clone(),MochigomaCollections::Pair(ms.clone(),mg.clone()))
					},
					UsiInitialPosition::Sfen(ref b,MochigomaCollections::Empty) => {
						(b.clone(),MochigomaCollections::Pair(HashMap::new(),HashMap::new()))
					}
				};
				(teban,banmen,mc,n,m)
			},
			_ => {
				panic!("invalid state.");
			}
		}
	};

	let start_time = Instant::now();

	let mvs:Vec<LegalMove> = Rule::legal_moves_all(&teban, &banmen, &mc);

	let count = process_moves(teban,&banmen,&mc,&mvs,DEPTH-1);

	let elapsed = start_time.elapsed();

	let count_scaled = count as u128 * 1000_000_000;
	let elapsed_scaled = elapsed.as_secs() as u128 * 1000_000_000 + elapsed.subsec_nanos() as u128;

	println!("{}.{:?}秒経過しました。", elapsed.as_secs(), elapsed.subsec_nanos() / 1_000_000);
	println!("{}個の指し手を生成しました。", count);
	println!("1秒あたり{}個の指し手を生成しました。", count_scaled / elapsed_scaled);

	println!("win_only_moves...");

	let start_time = Instant::now();

	let mut count = 0;

	count += Rule::win_only_moves(&teban, &banmen).len();
	let (c,t) = process_moves_with_win_only_moves(teban,&banmen,&mc,&mvs,DEPTH-1,count,start_time);
	count += c;

	let elapsed = t.duration_since(start_time);

	let count_scaled = count as u128 * 1000_000_000;
	let elapsed_scaled = elapsed.as_secs() as u128 * 1000_000_000 + elapsed.subsec_nanos() as u128;

	println!("{}.{:?}秒経過しました。", elapsed.as_secs(), elapsed.subsec_nanos() / 1_000_000);
	println!("{}個の指し手を生成しました。", count);
	println!("1秒あたり{}個の指し手を生成しました。", count_scaled / elapsed_scaled);

	println!("oute_only_moves...");

	let start_time = Instant::now();

	let mut count = 0;

	count += Rule::oute_only_moves_all(&teban, &banmen, &mc).len();
	let (c,t) = process_moves_with_oute_only_moves(teban,&banmen,&mc,&mvs,DEPTH-1,count,start_time);
	count += c;

	let elapsed = t.duration_since(start_time);

	let count_scaled = count as u128 * 1000_000_000;
	let elapsed_scaled = elapsed.as_secs() as u128 * 1000_000_000 + elapsed.subsec_nanos() as u128;

	println!("{}.{:?}秒経過しました。", elapsed.as_secs(), elapsed.subsec_nanos() / 1_000_000);
	println!("{}個の指し手を生成しました。", count);
	println!("1秒あたり{}個の指し手を生成しました。", count_scaled / elapsed_scaled);
}

fn process_moves(teban:Teban, banmen:&Banmen, mc:&MochigomaCollections, mvs:&Vec<LegalMove>, depth:u32) -> usize {
	if depth == 0 {
		return mvs.len();
	}

	let mut count = 0;

	for m in mvs {
		let next = Rule::apply_move_none_check(banmen,&teban,mc,&m.to_move());

		match next {
			(ref next,ref mc,_) => {
				let mvs:Vec<LegalMove> = Rule::legal_moves_all(&teban, next, mc);
				count += process_moves(teban.opposite(),next,mc,&mvs,depth-1);
			}
		}
	}

	count
}

fn process_moves_with_win_only_moves(teban:Teban,
									 banmen:&Banmen, mc:&MochigomaCollections,
									 mvs:&Vec<LegalMove>, depth:u32, count:usize, mut time:Instant) -> (usize,Instant) {
	if depth == 0 {
		return (count,time);
	}

	let mut count = 0;

	for m in mvs {
		let next = Rule::apply_move_none_check(banmen,&teban,mc,&m.to_move());

		match next {
			(ref next,ref mc,_) => {
				let mvs:Vec<LegalMove> = Rule::legal_moves_all(&teban, next, mc);
				let st = Instant::now();
				let count_win_only = Rule::win_only_moves(&teban, next).len();
				let elapsed = st.elapsed();
				time = time + elapsed;
				let (c,t) = process_moves_with_win_only_moves(teban.opposite(),next,mc,&mvs,depth-1,count_win_only,time);
				count += c;
				time = t;
			}
		}
	}

	(count,time)
}


fn process_moves_with_oute_only_moves(teban:Teban,
										banmen:&Banmen, mc:&MochigomaCollections,
										mvs:&Vec<LegalMove>, depth:u32, count:usize, mut time:Instant) -> (usize,Instant) {
	if depth == 0 {
		return (count,time);
	}

	let mut count = 0;

	for m in mvs {
		let next = Rule::apply_move_none_check(banmen,&teban,mc,&m.to_move());

		match next {
			(ref next,ref mc,_) => {
				let mvs:Vec<LegalMove> = Rule::legal_moves_all(&teban, next, mc);
				let st = Instant::now();
				let count_oute_only = Rule::oute_only_moves_all(&teban, next, mc).len();
				let elapsed = st.elapsed();
				time = time + elapsed;
				let (c,t) = process_moves_with_oute_only_moves(teban.opposite(),next,mc,&mvs,depth-1,count_oute_only,time);
				count += c;
				time = t;
			}
		}
	}

	(count,time)
}
