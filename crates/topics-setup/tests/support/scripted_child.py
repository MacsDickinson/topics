#!/usr/bin/env python3
"""Synthetic pinned app-server protocol, never a provider client."""
import json
import re
import sys
from pathlib import Path

SETTINGS = Path(@SETTINGS@)
LOG = Path(@LOG@)
FLAGS = @FLAGS@
if '--version' in sys.argv:
    print('codex-cli 0.165.0' if SETTINGS.read_text() == 'wrong_version' else 'codex-cli 0.166.0')
    sys.exit()


def emit(value):
    print(json.dumps(value), flush=True)


def result(identifier, value):
    emit({'id': identifier, 'result': value})


def call(identifier, tool, arguments):
    emit({'id': identifier, 'method': 'item/tool/call', 'params': {
        'threadId': 'thread', 'turnId': 'turn', 'callId': str(identifier),
        'namespace': 'topics', 'tool': tool, 'arguments': arguments}})
    reply = json.loads(sys.stdin.readline())
    assert reply['id'] == identifier
    assert reply['result']['success'], reply
    return reply['result']


def terminal(status):
    emit({'method': 'turn/completed', 'params': {
        'threadId': 'thread', 'turn': {'id': 'turn', 'status': status,
        'error': {'message': 'PRIVATE_ERROR_SENTINEL', 'details': 'PRIVATE_ERROR_SENTINEL'}}}})


for line in sys.stdin:
    request = json.loads(line)
    method, identifier, params = request.get('method'), request.get('id'), request.get('params', {})
    if method == 'initialize':
        result(identifier, {})
    elif method == 'config/read':
        config = {
            'openai_base_url': 'https://chatgpt.com/backend-api/codex',
            'chatgpt_base_url': 'https://chatgpt.com/backend-api/',
            'notify': [], 'model_provider': 'openai', 'skills': {'include_instructions': False},
            'agents': {'enabled': False}, 'analytics': {'enabled': False}, 'feedback': {'enabled': False},
            'compact_prompt': '', 'experimental_thread_store': {'type': 'local'},
            'otel': {'exporter': 'none', 'trace_exporter': 'none', 'metrics_exporter': 'none',
                     'log_user_prompt': False, 'log_agent_responses': False, 'log_guardian_assessments': False},
            'web_search': 'disabled', 'features': {flag: False for flag in FLAGS}, 'mcp_servers': {}}
        config['features']['multi_agent_v2'] = {'enabled': False, 'disable_native_tools': True}
        config['model_catalog_json'] = json.loads(next(arg.split('=', 1)[1] for arg in sys.argv if arg.startswith('model_catalog_json=')))
        result(identifier, {'config': config})
    elif method == 'account/read':
        result(identifier, {'account': {'type': 'chatgpt'}, 'requiresOpenaiAuth': True})
    elif method == 'thread/start':
        assert sorted(tool['name'] for tool in params['dynamicTools'][0]['tools']) == ['propose_changes', 'scope_sources']
        result(identifier, {'thread': {'id': 'thread', 'ephemeral': True}, 'cwd': params['cwd'],
                            'approvalPolicy': 'never', 'sandbox': {'type': 'readOnly', 'networkAccess': False},
                            'instructionSources': [], 'runtimeWorkspaceRoots': [], 'modelProvider': 'openai', 'model': 'gpt-6.1-sol'})
    elif method == 'mcpServerStatus/list':
        result(identifier, {'data': [], 'nextCursor': None})
    elif method == 'turn/start':
        result(identifier, {'turn': {'id': 'turn'}})
        if SETTINGS.read_text() == 'no_read':
            terminal('completed')
            continue
        prompt = params['input'][0]['text']
        scope_id = re.search(r'Scope ID: ([a-f0-9-]+)', prompt).group(1)
        response = call(100, 'scope_sources', {'scope_id': scope_id})
        context = json.loads(response['contentItems'][0]['text'])
        count = len(LOG.read_text().splitlines()) + 1 if LOG.exists() else 1
        with LOG.open('a') as stream:
            stream.write(json.dumps({'scope_id': scope_id, 'prompt': prompt, 'context': context}) + '\n')
        mode = SETTINGS.read_text()
        if mode != 'empty':
            citations = [source['citation'] for source in context['sources']]
            topics = [
                {'kind': 'topic', 'topic': {'id': '10000000-0000-4000-8000-000000000001', 'topic_kind': 'person',
                 'name': 'Synthetic Alex A42', 'aliases': ['employee A42'], 'sensitivity': 'unknown'}},
                {'kind': 'topic', 'topic': {'id': '10000000-0000-4000-8000-000000000002', 'topic_kind': 'project',
                 'name': 'Synthetic Ledger', 'aliases': [], 'sensitivity': 'unknown'}}]
            connection = {'kind': 'connection', 'connection': {'id': '10000000-0000-4000-8000-000000000003',
                'from': {'kind': 'topic', 'id': topics[0]['topic']['id']}, 'relation': 'leads',
                'to': {'kind': 'topic', 'id': topics[1]['topic']['id']}, 'uncertainty': None,
                'sources': citations, 'valid_from': None, 'valid_to': None}}
            if mode in ['pronouns', 'historic']:
                claims = []
                for index, source in enumerate(context['sources']):
                    historic = '2020-01-01' in source['content']
                    claims.append({'kind': 'claim', 'claim': {
                        'id': '30000000-0000-4000-8000-%012d' % (count * 100 + index * 2 + 1),
                        'revision_id': '30000000-0000-4000-8000-%012d' % (count * 100 + index * 2 + 2),
                        'subject_id': topics[0 if mode == 'pronouns' else 1]['topic']['id'],
                        'predicate': 'pronouns' if mode == 'pronouns' else 'leader',
                        'value': 'he/him' if mode == 'pronouns' else ('Taylor' if historic else 'Alex'),
                        'uncertainty': None, 'sources': [{'kind': 'passage', 'source': source['citation']}],
                        'supersedes': [], 'recorded_at': '2026-11-09T00:00:00Z',
                        'valid_from': None if mode == 'pronouns' else ('2020-01-01' if historic else '2026-10-01'),
                        'valid_to': None}})
                candidates = claims if mode == 'pronouns' else topics + claims
            elif mode == 'ambiguous':
                for topic in topics:
                    topic['topic']['name'] = 'Joe'
                    topic['topic']['topic_kind'] = 'person'
                topics[0]['topic']['aliases'] = ['employee A42']
                topics[1]['topic']['aliases'] = ['employee B93']
                candidates = topics + [{'kind': 'unresolved_identity', 'name': 'Joe',
                    'candidates': [topic['topic']['id'] for topic in topics], 'sources': citations}]
            else:
                candidates = topics + [connection]
            if mode == 'candidate_schema':
                topics[0]['topic']['private'] = 'PRIVATE_ERROR_SENTINEL'
            if mode == 'host_denied':
                connection['connection']['sources'][0]['passage_hash'] = '0' * 64
            if mode == 'bad_callback':
                call(99, 'scope_sources', {'scope_id': '00000000-0000-4000-8000-000000000000'})
            if mode in ['graph_claim', 'rehearsal_claim']:
                candidates.append({'kind': 'claim', 'claim': {
                    'id': '10000000-0000-4000-8000-000000000004',
                    'revision_id': '10000000-0000-4000-8000-000000000005',
                    'subject_id': topics[0]['topic']['id'],
                    'predicate': 'pronouns' if mode == 'rehearsal_claim' else 'responsibility',
                    'value': 'he/him' if mode == 'rehearsal_claim' else 'leads Ledger',
                    'uncertainty': None,
                    'sources': [{'kind': 'passage', 'source': source} for source in citations],
                    'supersedes': [], 'recorded_at': '2026-11-09T00:00:00Z',
                    'valid_from': None, 'valid_to': None}})
            if mode == 'split_calls':
                call(101, 'propose_changes', {'candidates': topics})
                call(102, 'propose_changes', {'candidates': [connection]})
            else:
                call(101, 'propose_changes', {'candidates': candidates})
        emit({'method': 'item/agentMessage/delta', 'params': {
            'threadId': 'thread', 'turnId': 'turn', 'itemId': 'text', 'delta': 'Synthetic turn buffered'}})
        if mode == 'malformed':
            print('PRIVATE_ERROR_SENTINEL', flush=True)
            sys.exit()
        if mode == 'hold_second' and count == 2:
            continue
        terminal('failed' if mode == 'fail_first' and count == 1 else 'completed')
    elif method == 'turn/interrupt':
        result(identifier, {})
        terminal('interrupted')
