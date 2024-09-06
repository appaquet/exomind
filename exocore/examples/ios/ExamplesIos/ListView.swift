import Combine
import SwiftUI
import Exocore

struct ListView: View {
    @EnvironmentObject var appState: AppState
    @ObservedObject var list: ItemList
    @State private var text = ""

    #if DEBUG
    init(mockedItems items: [Item]) {
        self.list = ItemList(defaultItems: items)
    }
    #endif

    init() {
        self.list = ItemList()
    }

    var body: some View {
        VStack {
            HStack {
                Button("Rejoin") {
                    self.appState.forceDiscovery = true
                    self.appState.triggerChanged()
                }
                Button("Connect") {
                    self.list.connect(appState: self.appState)
                }
                Button("Disconnect") {
                    self.list.drop()
                }
            }.padding()

            HStack {
                TextField("", text: $text)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .padding()

                Button("Add") {
                    self.list.add(self.$text.wrappedValue)
                    self.$text.wrappedValue = ""
                }.padding()
            }

            List {
                ForEach(list.items, id: \.self.id) { (item) in
                    Text(item.text)
                }.onDelete { (indexSet) in
                    self.list.remove(atOffsets: indexSet)
                }
            }
        }.onAppear {
            self.list.connect(appState: self.appState)
        }
    }
}

class ItemList: ObservableObject {
    var resultStream: QueryStreamHandle?

    @Published var items: [Item] = []

    init(defaultItems items: [Item]? = nil) {
        if let items = items {
            self.items = items
        }
    }

    func connect(appState: AppState) {
        let query = QueryBuilder.withTrait(Exocore_Test_TestMessage.self)
                .count(100)
                .build()
        self.resultStream = ExocoreClient.store.watchedQuery(query: query, onChange: { [weak self] (status, results) in
            DispatchQueue.main.async {
                if let results = results {
                    self?.items = results.entities.map { (result: Exocore_Store_EntityResult) -> Item in

                        var title = "UNKNOWN"
                        if let trt = result.entity.traitOfType(Exocore_Test_TestMessage.self) {
                            title = trt.message.string1
                        }

                        return Item(id: result.entity.id, text: title)
                    }
                } else {
                    self?.items = []
                }

            }
        })
    }

    func add(_ text: String) {
        var msg = Exocore_Test_TestMessage()
        msg.string1 = text

        let mutation = try! MutationBuilder
                .createEntity()
                .putTrait(message: msg)
                .build()

        ExocoreClient.store.mutate(mutation: mutation)
    }

    func remove(atOffsets: IndexSet) {
        let item = self.items[atOffsets.first!]

        let mutation = MutationBuilder
                .updateEntity(entityId: item.id)
                .deleteEntity()
                .build()

        ExocoreClient.store.mutate(mutation: mutation)
    }

    func drop() {
        self.items = []
        self.resultStream = nil
    }
}

struct Item: Decodable, Identifiable {
    var id: String
    var text: String
}

#if DEBUG
struct SwiftUIView_Previews: PreviewProvider {
    static var previews: some View {
        ListView(mockedItems: [
            Item(id: "1", text: "Item 1"),
            Item(id: "2", text: "Item 2"),
        ]).environmentObject(AppState())
    }
}
#endif
