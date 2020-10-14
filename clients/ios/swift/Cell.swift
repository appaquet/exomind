import Foundation

public class Cell {
    weak var client: ClientInstance?

    init(client: ClientInstance) {
        self.client = client
    }

    public func generateAuthToken(expirationDays: Int? = nil) -> String? {
        guard let context = self.client?.context else { return nil }

        let days = expirationDays ?? 0
        let tokenPtr = exocore_generate_auth_token(context, UInt(days))
        let tokenStr = String(cString: tokenPtr!)
        exocore_free_string(tokenPtr)

        if tokenStr == "" {
            return nil
        } else {
            return tokenStr
        }
    }
}
