#!/usr/bin/env python3
from datetime import datetime, timedelta

# Test dates from TradingView
test_dates = [
    ("MSFT", "2025-10-28"),
    ("AMZN", "2025-10-23"),
    ("META", "2025-10-22"),
    ("TSLA", "2025-10-15"),
    ("LLY", "2025-10-30"),  # From debug file
]

current_date = datetime.now()
print(f"Current date: {current_date.strftime('%Y-%m-%d')}")
print("\nExpected days until announcement:")
print("-" * 40)

for symbol, date_str in test_dates:
    try:
        target_date = datetime.strptime(date_str, "%Y-%m-%d")
        days = (target_date - current_date).days
        print(f"{symbol}: {date_str} → {days} days")
    except Exception as e:
        print(f"{symbol}: {date_str} → ERROR: {e}")

print("\n✅ All dates should show different values, not all 100!")
print("Each stock should have a unique number of days until earnings.")