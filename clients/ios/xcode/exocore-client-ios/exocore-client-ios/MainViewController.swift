import Combine
import SwiftUI

struct MainView: View {
    @ObservedObject var list: MyList
    @State private var text = ""

    init(mockedItems items: [Item]) {
        self.list = MyList()
    }

    init() {
        self.list = MyList()
    }

    var body: some View {
        VStack {
            HStack {
                Button("Watch") {
                    self.list.watch()
                }
                Button("Unwatch") {
                    self.list.unwatch()
                }
                Button("Disconnect") {
                    self.list.drop()
                }
            }

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
        }
    }
}

class MyList: ObservableObject {
    var client: EXOClient?
    var resultStream: EXOQueryStreamHandle?

    @Published var items: [Item] = []

    init() {
    }

    func watch() {
        if self.client == nil {
            let config = """
                         keypair: ae4WbDdfhv3416xs8S2tQgczBarmR8HKABvPCmRcNMujdVpDzuCJVQADVeqkqwvDmqYUUjLqv7kcChyCYn8R9BNgXP
                         public_key: pe5ZG43uAcfLxYSGaQgj1w8hQT4GBchEVg5mS2b1EfXcMb
                         name: ios
                         path: ""
                         listen_addresses:
                           - /ip4/0.0.0.0/tcp/0
                         cells:
                           - location:
                               Instance:
                                 public_key: pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ
                                 keypair: ae55Nfv11ppyFVxCDaYovcxTcaTDaSzSFjiVoiC3VwGARfEuaqGcgoJUdVpqfwKQVDN4rvGKUvt4yqQc6w7it7PCpG
                                 name: ""
                                 path: "./cell"
                                 nodes:
                                   - node:
                                       public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP
                                       addresses:
                                         - /ip4/192.168.2.13/tcp/3330
                                         - /ip4/192.168.2.16/tcp/3330
                                         - /ip4/192.168.2.19/tcp/3330
                                       name: server
                                     roles:
                                       - 1
                                       - 2
                                   - node:
                                       public_key: pe5ZG43uAcfLxYSGaQgj1w8hQT4GBchEVg5mS2b1EfXcMb
                                       addresses: []
                                       name: ios
                                     roles: []
                                   - node:
                                       public_key: peDEpWCwRyMA6QVtz1wBdfKWx8xTYC8CVerF7pSTUsxfxc
                                       addresses: []
                                       name: web
                                     roles: []
                                 apps: []

                         """

            self.client = try? EXOClient(yamlConfig: config)
        }

        let query = EXOQueryBuilder.withTrait(message: Exocore_Test_TestMessage())
                .count(count: 100)
                .build()
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

    func add(_ text: String) {
        var msg = Exocore_Test_TestMessage()
        msg.string1 = text

        let mutation = try! EXOMutationBuilder
                .createEntity()
                .putTrait(trait: msg)
                .build()

        _ = self.client?.mutate(mutation: mutation, onCompletion: { (status, res) in
        })
    }

    func remove(atOffsets: IndexSet) {
        let item = self.items[atOffsets.first!]

        let mutation = EXOMutationBuilder
                .updateEntity(entityId: item.id)
                .deleteTrait(traitId: "")
                .build()

        _ = self.client?.mutate(mutation: mutation, onCompletion: { (status, res) in
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

