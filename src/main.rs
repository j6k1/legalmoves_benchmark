extern crate usiagent;

use std::time::{Instant};

use usiagent::rule::*;
use usiagent::protocol::*;
use usiagent::shogi::*;

const DEPTH:u32 = 4;
const INITIAL_SFEN:&'static str = "sfen l6nl/5+P1gk/2np1S3/p1p4Pp/3P2Sp1/1PPb2P1P/P5GS1/R8/LN4bKL w RGgsn5p 1";

fn main() {
	let position_parser = PositionParser::new();

	let (teban, banmen, mc, _, _) = match  position_parser.parse(&INITIAL_SFEN.split(" ").collect::<Vec<&str>>()) {
		Ok(position) => {
			position.extract()
		},
		_ => {
			panic!("invalid state.");
		}
	};

	let state = State::new(banmen);

	let start_time = Instant::now();

	let mvs:Vec<LegalMove> = Rule::legal_moves_all(teban, &state, &mc);

	let count = process_moves(teban,&state,&mc,&mvs,DEPTH-1);

	let elapsed = start_time.elapsed();

	let count_scaled = count as u128 * 1000_000_000;
	let elapsed_scaled = elapsed.as_secs() as u128 * 1000_000_000 + elapsed.subsec_nanos() as u128;

	println!("{}.{:?}秒経過しました。", elapsed.as_secs(), elapsed.subsec_nanos() / 1_000_000);
	println!("{}個の指し手を生成しました。", count);
	println!("1秒あたり{}個の指し手を生成しました。", count_scaled / elapsed_scaled);

	println!("win_only_moves...");

	let start_time = Instant::now();

	let mut count = 0;

	count += Rule::win_only_moves(teban, &state).len();
	let (c,t) = process_moves_with_win_only_moves(teban,&state,&mc,&mvs,DEPTH-1,count,start_time);
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

	count += Rule::oute_only_moves_all(teban, &state, &mc).len();
	let (c,t) = process_moves_with_oute_only_moves(teban,&state,&mc,&mvs,DEPTH-1,count,start_time);
	count += c;

	let elapsed = t.duration_since(start_time);

	let count_scaled = count as u128 * 1000_000_000;
	let elapsed_scaled = elapsed.as_secs() as u128 * 1000_000_000 + elapsed.subsec_nanos() as u128;

	println!("{}.{:?}秒経過しました。", elapsed.as_secs(), elapsed.subsec_nanos() / 1_000_000);
	println!("{}個の指し手を生成しました。", count);
	println!("1秒あたり{}個の指し手を生成しました。", count_scaled / elapsed_scaled);

	println!("is_nyugyoku_win...");

	let start_time = Instant::now();

	let mut count = 0;

	let _ = Rule::is_nyugyoku_win(&state, teban, &mc, &None);
	let c = process_moves_with_is_nyugyoku_win(teban,&state,&mc,&mvs,DEPTH-1);

	count += c;

	let elapsed = start_time.elapsed();

	let count_scaled = count as u128 * 1000_000_000;
	let elapsed_scaled = elapsed.as_secs() as u128 * 1000_000_000 + elapsed.subsec_nanos() as u128;

	println!("{}.{:?}秒経過しました。", elapsed.as_secs(), elapsed.subsec_nanos() / 1_000_000);
	println!("{}個の指し手を生成しました。", count);
	println!("1秒あたり{}個の指し手を生成しました。", count_scaled / elapsed_scaled);
}

fn process_moves(teban:Teban, state:&State, mc:&MochigomaCollections, mvs:&Vec<LegalMove>, depth:u32) -> usize {
	if depth == 0 {
		return mvs.len();
	}

	let mut count = 0;

	for m in mvs {
		let next = Rule::apply_move_none_check(state,teban,mc,m.to_applied_move());

		match next {
			(ref next,ref mc,_) => {
				let mvs:Vec<LegalMove> = Rule::legal_moves_all(teban.opposite(), next, mc);
				count += process_moves(teban.opposite(),next,mc,&mvs,depth-1);
			}
		}
	}

	count
}

fn process_moves_with_win_only_moves(teban:Teban,
									 state:&State, mc:&MochigomaCollections,
									 mvs:&Vec<LegalMove>, depth:u32, count:usize, mut time:Instant) -> (usize,Instant) {
	if depth == 0 {
		return (count,time);
	}

	let mut count = 0;

	for m in mvs {
		let next = Rule::apply_move_none_check(state,teban,mc,m.to_applied_move());

		match next {
			(ref next,ref mc,_) => {
				let mvs:Vec<LegalMove> = Rule::legal_moves_all(teban.opposite(), next, mc);
				let st = Instant::now();
				let count_win_only = Rule::win_only_moves(teban.opposite(), next).len();
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
										state:&State, mc:&MochigomaCollections,
										mvs:&Vec<LegalMove>, depth:u32, count:usize, mut time:Instant) -> (usize,Instant) {
	if depth == 0 {
		return (count,time);
	}

	let mut count = 0;

	for m in mvs {
		let next = Rule::apply_move_none_check(state,teban,mc,m.to_applied_move());

		match next {
			(ref next,ref mc,_) => {
				let mvs:Vec<LegalMove> = Rule::legal_moves_all(teban.opposite(), next, mc);
				let st = Instant::now();
				let count_oute_only = Rule::oute_only_moves_all(teban.opposite(), next, mc).len();
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

fn process_moves_with_is_nyugyoku_win(teban:Teban,
								   state:&State, mc:&MochigomaCollections,
								   mvs:&Vec<LegalMove>, depth:u32) -> usize {
	if depth == 0 {
		return mvs.len();
	}

	let mut count = 0;

	for m in mvs {
		let next = Rule::apply_move_none_check(state,teban,mc,m.to_applied_move());

		match next {
			(ref next,ref mc,_) => {
				let _ = Rule::is_nyugyoku_win(next,teban.opposite(),mc,&None);
				let mvs:Vec<LegalMove> = Rule::legal_moves_all(teban.opposite(), next, mc);
				count += process_moves_with_is_nyugyoku_win(teban.opposite(),next,mc,&mvs,depth-1);
			}
		}
	}

	count
}
