import { exocore, Exocore, matchTrait, MutationBuilder, QueryBuilder } from 'exocore';
import React from 'react';

export default class List extends React.Component {
    constructor(props) {
        super(props);

        this.state = {entities: []};

        this.registerQuery();
    }

    render() {
        return (
            <div>
                <Input onAdd={this.onAdd.bind(this)}/>

                <ul>
                    {this.renderList()}
                </ul>
            </div>
        );
    }

    renderList() {
        const DeleteButton = (props) => {
            return <button onClick={this.onDelete.bind(this, props.entity.id, props.trait.id)}>Delete</button>
        };

        return this.state.entities.map(res =>
            <li key={res.entity.id}>{res.message.string1} (<DeleteButton entity={res.entity} trait={res.trait}/>)</li>
        );
    }

    async onAdd(text) {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exocore.test.TestMessage({
                string1: text,
            }))
            .build();

        await Exocore.store.mutate(mutation);
    }

    async onDelete(entityId, traitId) {
        const mutation = MutationBuilder
            .updateEntity(entityId)
            .deleteTrait(traitId)
            .build();

        await Exocore.store.mutate(mutation);
    }

    registerQuery() {
        const query = QueryBuilder
            .withTrait(exocore.test.TestMessage)
            .count(100)
            .build();

        this.watched_query = Exocore.store.watchedQuery(query).onChange((results) => {
            let res = results.entities.flatMap((res) => {
                return matchTrait(res.entity.traits[0], {
                    [Exocore.registry.messageFullName(exocore.test.TestMessage)]: (trait, message) => {
                        return {entity: res.entity, trait: trait, message: message};
                    }
                })
            });

            this.setState({
                entities: res
            })
        });
    }

    componentWillUnmount() {
        this.watched_query.free();
    }
}

class Input extends React.Component {
    constructor(props) {
        super(props);

        this.state = {
            text: ''
        }
    }

    render() {
        return (
            <div>
                <input value={this.state.text} onChange={this.onTextChange.bind(this)}/>
                <button onClick={this.onAddClick.bind(this)}>Add</button>
            </div>
        )
    }

    onTextChange(e) {
        this.setState({
            text: e.target.value
        });
    }

    onAddClick(e) {
        this.props.onAdd(this.state.text);
        this.setState({
            text: ''
        });
    }
}

