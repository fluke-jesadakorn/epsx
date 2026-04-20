import {
  getChatAttachments,
  normalizeChatMessage,
  normalizeChatMessageMetadata,
} from '@/shared/api/chat';

describe('getChatAttachments', () => {
  it('returns an empty array when metadata is null', () => {
    expect(getChatAttachments(null)).toEqual([]);
  });

  it('returns an empty array when attachments are missing', () => {
    expect(getChatAttachments({})).toEqual([]);
  });

  it('returns an empty array when attachments are null', () => {
    expect(getChatAttachments({ attachments: null } as any)).toEqual([]);
  });

  it('returns an empty array when metadata is a primitive', () => {
    expect(getChatAttachments('bad-payload' as any)).toEqual([]);
  });

  it('filters malformed attachments out of the response payload', () => {
    expect(
      getChatAttachments({
        attachments: [
          {
            url: 'https://cdn.epsx.io/chat/1/file.png',
            filename: 'file.png',
            file_type: 'image/png',
            size: 2048,
          },
          {
            url: 'https://cdn.epsx.io/chat/1/bad.png',
            filename: 'bad.png',
          },
        ],
      } as any)
    ).toEqual([
      {
        url: 'https://cdn.epsx.io/chat/1/file.png',
        filename: 'file.png',
        file_type: 'image/png',
        size: 2048,
      },
    ]);
  });
});

describe('normalizeChatMessageMetadata', () => {
  it('coerces null metadata to an empty object', () => {
    expect(normalizeChatMessageMetadata(null)).toEqual({});
  });

  it('keeps non-attachment metadata fields while sanitizing attachments', () => {
    expect(
      normalizeChatMessageMetadata({
        note: 'hello',
        attachments: [
          {
            url: 'https://cdn.epsx.io/chat/1/file.png',
            filename: 'file.png',
            file_type: 'image/png',
            size: 2048,
          },
          { filename: 'broken.png' },
        ],
      } as any)
    ).toEqual({
      note: 'hello',
      attachments: [
        {
          url: 'https://cdn.epsx.io/chat/1/file.png',
          filename: 'file.png',
          file_type: 'image/png',
          size: 2048,
        },
      ],
    });
  });
});

describe('normalizeChatMessage', () => {
  it('replaces null metadata so direct attachment access is safe', () => {
    const message = normalizeChatMessage({
      id: 'msg-1',
      conversation_id: 'conv-1',
      sender_type: 'user',
      sender_address: null,
      content: 'hello',
      is_read: false,
      metadata: null,
      created_at: '2026-04-19T00:00:00.000Z',
    });

    expect(message.metadata).toEqual({});
    expect(message.metadata?.attachments).toBeUndefined();
  });
});
