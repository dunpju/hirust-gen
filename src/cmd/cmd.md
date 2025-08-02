cargo run gen code --out="./errcode/mod.rs" --source="./yaml"
cargo run gen enum --out="./enums" --name="-e=order_flow -f=发起订单:selling_assistant-1-销售内勤,sale-2-销售"
cargo run gen enum --out="./enums" --file="./src/cmd/enum.md"
