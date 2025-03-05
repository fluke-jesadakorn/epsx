// Main application module
import { Module, MiddlewareConsumer, NestModule } from "@nestjs/common";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { MongooseModule } from "@nestjs/mongoose";
import cookieParser from "cookie-parser";
import { AuthModule } from "./modules/auth/auth.module";
import { MarketModule } from "./modules/market/market.module";

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
    MongooseModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: async (configService: ConfigService) => ({
        uri: configService.get<string>("MONGODB_URI"),
      }),
      inject: [ConfigService],
    }),
    AuthModule,
    MarketModule,
  ],
})
export class AppModule implements NestModule {
  constructor(private configService: ConfigService) {}

  configure(consumer: MiddlewareConsumer) {
    // Use cookie parser middleware
    consumer.apply(cookieParser()).forRoutes("*");

    // Log environment variables for debugging
    console.log('Backend URL:', this.configService.get('BACKEND_URL'));
    console.log('Frontend URL:', this.configService.get('FRONTEND_URL'));
  }
}
