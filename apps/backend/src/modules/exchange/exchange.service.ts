import { Injectable, NotFoundException } from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import { Model } from "mongoose";
import axios from "axios";
import * as cheerio from "cheerio";
import { Exchange, IExchange } from "@epsx/shared";
import { ExchangeResponseDto, PaginatedExchangeResponse } from "./dto/exchange.dto";

@Injectable()
export class ExchangeService {
  constructor(
    @InjectModel(Exchange.name)
    private readonly exchangeModel: Model<IExchange>
  ) {}

  // Create
  async create(exchangeData: Partial<IExchange>): Promise<ExchangeResponseDto> {
    const existingExchange = await this.exchangeModel
      .findOne({ market_code: exchangeData.market_code })
      .exec();

    if (existingExchange) {
      throw new Error(
        `Exchange with market code ${exchangeData.market_code} already exists`
      );
    }

    const exchange = new this.exchangeModel(exchangeData);
    const saved = await exchange.save();
    return {
      ...saved.toObject(),
      _id: saved._id.toString()
    };
  }

  // Read
  async findAll(skip = 0, limit = 10): Promise<PaginatedExchangeResponse> {
    try {
      // Remove console logs in production
      const [data, total] = await Promise.all([
        this.exchangeModel.find().skip(skip).limit(limit).lean().exec(),
        this.exchangeModel.countDocuments().exec(),
      ]).catch((err) => {
        console.error("MongoDB operation failed:", err);
        throw err;
      });

      if (!data || data.length === 0) {
        // Seed initial data if no exchanges exist
        // TODO: Use a separate script or migration for data seeding
        const sampleExchange = {
          exchange_name: "New York Stock Exchange",
          country: "United States",
          market_code: "NYSE",
          currency: "USD",
          exchange_url: "https://www.nyse.com",
          timezone: "America/New_York",
        };

        await this.create(sampleExchange);

        // Fetch again after seeding
        const [newData, newTotal] = await Promise.all([
          this.exchangeModel.find().skip(skip).limit(limit).lean().exec(),
          this.exchangeModel.countDocuments().exec(),
        ]);

        return {
          items: newData.map(doc => ({
            ...doc,
            _id: doc._id.toString()
          })),
          total: newTotal,
          page: Math.floor(skip / limit) + 1,
          limit
        };
      }

      return {
        items: data.map(doc => ({
          ...doc,
          _id: doc._id.toString()
        })),
        total,
        page: Math.floor(skip / limit) + 1,
        limit
      };
    } catch (error) {
      console.error("Error in findAll:", error);
      // TODO: Implement more robust error handling and logging
      // Consider implementing a custom error class for database errors
      throw new Error(`Failed to fetch exchanges: ${error}`);
    }
  }

  async findOne(marketCode: string): Promise<ExchangeResponseDto> {
    const exchange = await this.exchangeModel
      .findOne({ market_code: marketCode })
      .exec();
    if (!exchange) {
      throw new NotFoundException(
        `Exchange with market code ${marketCode} not found`
      );
    }
    return {
      ...exchange.toObject(),
      _id: exchange._id.toString()
    };
  }

  // Update
  async update(marketCode: string, updateData: Partial<IExchange>): Promise<ExchangeResponseDto> {
    const exchange = await this.exchangeModel
      .findOneAndUpdate({ market_code: marketCode }, updateData, { new: true })
      .exec();

    if (!exchange) {
      throw new NotFoundException(
        `Exchange with market code ${marketCode} not found`
      );
    }
    return {
      ...exchange.toObject(),
      _id: exchange._id.toString()
    };
  }

  // Delete
  async remove(marketCode: string): Promise<void> {
    const exchange = await this.exchangeModel
      .findOneAndDelete({ market_code: marketCode })
      .exec();

    if (!exchange) {
      throw new NotFoundException(
        `Exchange with market code ${marketCode} not found`
      );
    }
    return;
  }

  // Web scraping functionality using Cheerio
  async scrapeAndSaveExchanges(): Promise<ExchangeResponseDto[]> {
    try {
      // Fetch the HTML content
      const response = await axios.get(
        "https://stockanalysis.com/list/exchanges/",
        {
          headers: {
            "User-Agent":
              "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
          },
        }
      );

      // Load the HTML content into Cheerio
      const $ = cheerio.load(response.data);

      // Find the table and extract data
      const exchanges: Array<{
        exchange_name: string;
        country: string;
        market_code: string;
        currency: string;
        exchange_url: string;
        timezone: string;
      }> = [];

      // TODO: Use a more robust selector for table rows
      // Consider using a more specific selector to avoid issues with website changes
      $("#main > div > div > table tbody tr").each((_, row) => {
        const cells = $(row).find("td");
        const nameCell = $(cells[0]);

        exchanges.push({
          exchange_name: nameCell.text().trim(),
          country: $(cells[1]).text().trim(),
          market_code: $(cells[2]).text().trim(),
          currency: $(cells[3]).text().trim(),
          exchange_url: nameCell.find("a").attr("href")?.trim() || "",
          // TODO: Dynamically determine timezone based on exchange location
          timezone: "UTC",
        });
      });

      if (exchanges.length === 0) {
        throw new Error("No exchanges found");
      }

      let newCount = 0;
      let updateCount = 0;

      for (const exchangeData of exchanges) {
        try {
          const existingExchange = await this.exchangeModel
            .findOne({ market_code: exchangeData.market_code })
            .exec();

          if (existingExchange) {
            await this.update(exchangeData.market_code, exchangeData);
            updateCount++;
          } else {
            await this.create(exchangeData);
            newCount++;
          }
        } catch (error) {
          console.error(
            `Failed to save exchange ${exchangeData.market_code}:`,
            error
          );
          // TODO: Implement more robust error handling for individual exchange saves
        }
      }

      const allExchanges = await this.exchangeModel.find().exec();
      return allExchanges.map(exchange => ({
        ...exchange.toObject(),
        _id: exchange._id.toString()
      }));
    } catch (error) {
      console.error("Failed to scrape exchanges:", error);
      throw error;
    }
  }
}
