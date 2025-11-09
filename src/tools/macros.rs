

// Unit test for the proc-macro-generated Tool wrapper
#[cfg(test)]
mod tests {
	use serde_json::json;
	use futures::executor::block_on;
	use crate::tools::traits::Tool;

	// Use the proc-macro attribute to generate the Tool implementation (use crate-local re-export
	// so the macro expands with `crate` references when run inside the library tests)
	#[crate::tool(
		name = "get_weather",
		description = "Get weather for a given city",
		params(city = "City name, e.g. 'San Francisco'")
	)]
	fn get_weather(city: String) -> String {
		format!("It's always sunny in {}!", city)
	}

	#[test]
	fn proc_macro_generated_tool_runs() {
		let tool = GetWeatherTool;
		let args = json!({ "city": "sf" });
		let fut = tool.run(args);
		let got = block_on(fut).expect("tool run failed");
		assert_eq!(got, "It's always sunny in sf!");
	}
}

 