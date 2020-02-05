
import Combine
import SwiftUI

struct MainView: View {
    @ObservedObject var list: MyList

    init(mockedItems items: [Item]) {
        self.list = MyList()
    }

    init() {
        self.list = MyList()
    }

    var body: some View {
        VStack {
            HStack {
                Button("Watch query") {
                    self.list.watch()
                }
                Button("Unwatch") {
                    self.list.unwatch()
                }
                Button("Disconnect") {
                    self.list.drop()
                }
            }

            List(list.items) { item in
                Text(item.text)
            }
        }
    }
}

class MyList : ObservableObject {
    var client: Client?
    var resultStream: ResultStream?

    @Published var items: [Item] = []

    init() {
    }

    func watch() {
        if self.client == nil {
            self.client = Client()
        }

        var query = Exocore_Index_EntityQuery()
        var match = Exocore_Index_MatchPredicate()
        match.query = "test"
        query.match = match

        self.resultStream = self.client?.watched_query(query: query, onChange: { [weak self] (status, results) in
            DispatchQueue.main.async {
                if let results = results {
                    self?.items = results.entities.map { (result: Exocore_Index_EntityResult) -> Item in

                        var title = "UNKNOWN"
                        if let trait = result.entity.traits.first {
                            if trait.message.isA(Exocore_Test_TestMessage.self) {
                                let msg = try! Exocore_Test_TestMessage(unpackingAny: trait.message)
                                title = msg.string1
                            }
                        }

                        return Item(id: result.entity.id, text: title)
                    }
                } else {
                    self?.items = []
                }

            }
        })
    }

    func unwatch() {
        self.resultStream = nil
    }

    func drop() {
        self.client = nil
    }

}

struct Item: Decodable, Identifiable {
    var id: String

    var text: String
}

struct SwiftUIView_Previews: PreviewProvider {
    static var previews: some View {
        MainView(mockedItems: [Item(id: "123", text: "Hello")])
    }
}

class MainViewController: UIHostingController<MainView> {
    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder, rootView: MainView())
    }
}

