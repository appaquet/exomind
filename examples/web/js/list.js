import React from 'react';

export default class List extends React.Component {
    constructor(props) {
        super(props);

        this.exocore = props.exocore;
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
        return this.state.entities.map(entity =>
            <li key={entity.id}>{entity.traits[0].title}</li>
        );
    }

    async onAdd(text) {
        await this.exocore.mutate.create_entity("exocore.task", {
            title: text
        }).execute();
    }

    async fetchList() {
        let result = this.exocore.query
            .with_trait("exocore.task")
            .with_count(1000)
            .execute();
        await result.ready();

        let results = result.to_json();
        result.free();

        this.setState({
            entities: results.results.map(result => {
                return result.entity;
            })
        })
    }

    registerQuery() {
        this.watched_query = this.exocore.query
            .with_trait("exocore.task")
            .with_count(1000)
            .execute_and_watch();

        this.watched_query.on_change(() => {
            let results = this.watched_query.to_json();
            this.setState({
                entities: results.results.map(result => {
                    return result.entity;
                })
            })
        })
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

