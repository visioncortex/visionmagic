import * as simplificationAndSegmentation from "./lib/impression";
import * as repair from "./lib/repair";

document.body.id == 'repair' ? repair.main() : simplificationAndSegmentation.main();