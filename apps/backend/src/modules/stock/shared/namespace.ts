export const Deps = {
  NestCommon: require("@nestjs/common"),
  NestMongoose: require("@nestjs/mongoose"),
  Mongoose: require("mongoose"),
  SharedModels: require("@epsx/shared"),
  StockTransformers: require("../transformers/stock.transformer"),
  HttpUtils: require("../services/http.service")
};
