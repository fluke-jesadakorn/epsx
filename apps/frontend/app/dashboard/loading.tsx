export default function Loading() {
  return (
    <div className="container mx-auto py-8">
      <div className="max-w-4xl mx-auto">
        <div className="bg-card rounded-lg shadow-sm p-6 space-y-4 animate-pulse">
          <div className="h-8 w-48 bg-muted rounded" />
          
          <div className="space-y-4">
            <div className="p-4 bg-muted rounded-md space-y-3">
              <div className="h-6 w-32 bg-background rounded" />
              <div className="space-y-2">
                <div className="h-4 w-full max-w-[200px] bg-background rounded" />
                <div className="h-4 w-full max-w-[180px] bg-background rounded" />
                <div className="h-4 w-full max-w-[160px] bg-background rounded" />
              </div>
            </div>

            <div className="h-4 w-full max-w-[300px] bg-muted rounded" />
          </div>
        </div>
      </div>
    </div>
  );
}
