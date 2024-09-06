import { exocore, Exocore, matchTrait, MutationBuilder, QueryBuilder, WatchedQueryWrapper } from 'exocore';
import React, { ChangeEvent } from 'react';

interface IListProps {
}

interface IListItem {
    entity: exocore.store.IEntity;
    trait: exocore.store.ITrait;
    message: exocore.test.ITestMessage;
}

interface IListState {
    items: IListItem[];
    loading: boolean;
}

export default class List extends React.Component<IListProps, IListState> {
    private watchedQuery: WatchedQueryWrapper;

    constructor(props: IListProps) {
        super(props);

        this.state = {
            items: [],
            loading: true,
        };

        this.registerQuery();
    }

    render() {
        return (
            <div>
                <Input onAdd={this.onAdd} />

                {this.state.loading && <div className="loading">Loading...</div>}

                <ul>
                    {this.renderList()}
                </ul>
            </div>
        );
    }

    renderList() {
        const DeleteButton = (props: { item: IListItem }) => {
            return <button onClick={() => this.onDelete(props.item)}>Delete</button>
        };

        return this.state.items.map(item =>
            <li key={item.entity.id} className="item"><span className="text">{item.message.string1}</span> (<DeleteButton item={item} />)</li>
        );
    }

    onAdd = async (text: string) => {
        this.setState({ loading: true });

        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exocore.test.TestMessage({
                string1: text,
            }))
            .build();

        await Exocore.store.mutate(mutation);
    }

    async onDelete(item: IListItem) {
        const mutation = MutationBuilder
            .updateEntity(item.entity.id)
            .deleteTrait(item.trait.id)
            .build();

        await Exocore.store.mutate(mutation);
    }

    registerQuery() {
        const query = QueryBuilder
            .withTrait(exocore.test.TestMessage)
            .count(100)
            .build();

        this.watchedQuery = Exocore.store.watchedQuery(query).onChange((results) => {
            let res = results.entities.flatMap((res) => {
                return matchTrait(res.entity.traits[0], {
                    [Exocore.registry.messageFullName(exocore.test.TestMessage)]: (trait, message) => {
                        return { entity: res.entity, trait: trait, message: message };
                    }
                })
            });

            this.setState({
                items: res,
                loading: false,
            })
        });
    }

    componentWillUnmount() {
        this.watchedQuery.free();
    }
}

interface IInputProps {
    onAdd: (text: string) => void;
}

interface IInputState {
    text: string;
}

class Input extends React.Component<IInputProps, IInputState> {
    constructor(props: IInputProps) {
        super(props);

        this.state = {
            text: ''
        }
    }

    render() {
        return (
            <div>
                <input value={this.state.text} onChange={this.onTextChange} id="input-text" />
                <button onClick={this.onAddClick} id="input-add">Add</button>
            </div>
        )
    }

    onTextChange = (e: ChangeEvent<HTMLInputElement>) => {
        this.setState({
            text: e.target.value
        });
    }

    onAddClick = () => {
        this.props.onAdd(this.state.text);

        this.setState({
            text: ''
        });
    }
}
