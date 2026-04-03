

**A User Guide to GenAI API** 

**Revision:** 3.4

**Change History**

| Date | Version | Author | Changes |
| :---- | :---- | :---- | :---- |
| 08-22-2023 | 1.0 | Dipjyoti Bisharad | Initial Draft |
| 08-24-2023 | 1.1 | Jahnavi Alike | Updated sessionId usage process. |
| 12-12-2023 | 1.2 | Sunil Vurandur | Updated details on different model types |
| 03-28-2024 | 1.3 | Jahnavi Alike | Text rearrangement |
| 04-23-2024 | 1.4 | Dipjyoti Bisharad | Introduced additional args field in response |
| 05-08-2024 | 2.0 | Dipjyoti Bisharad | Updated to v2 of chat API |
| 05-17-2024 | 2.1 | Dipjyoti Bisharad | Added Gemini 1.5 Pro |
| 06-25-2024 | 2.2 | Dipjyoti Bisharad | Added disclaimer on omitting the userId from payload |
| 06-27-2024 | 2.3 | Dipjyoti Bisharad | Added the datastoreId to the request payload. Introduced GPT 4 Omni, Claude Sonnet. Replaced Gemini 1.5 Pro with Gemini 1.5 Flash. Deprecation of legacy /chat. |
| 07-10-2024 | 2.4 | Dipjyoti Bisharad | Added modelConfig in the chat payload. Added application identifier x-msi-genai-client in header. Upload files to a chat session |
| 08-09-2024 | 2.5 | Dipjyoti Bisharad | Added note regarding impersonation with API keys |
| 08-12-2024 | 2.6 | Dipjyoti Bisharad | Added userId usage for the /upload endpoint |
| 09-16-2024 | 2.7 | Dipjyoti Bisharad | Added endpoints for delete message and retrieve session messages |
| 11-15-2024 | 2.8 | Dipjyoti Bisharad | Updated contacts |
| 11-18-2024 | 2.9 | Girish Manivel | Added API error codes |
| 01-09-2025 | 3.0 | Anjali Kamath | Added Init Datastore API details |
| 01-15-2025 | 3.1 | Vibin Jacob | Added default Model config values |
| 01-30-2025 | 3.2 | Vibin Jacob | Added Nova Lite |
| 03-04-2025 | 3.3 | Anjali Kamath | Updated Model details |
| 03-27-2025 | 3.4 | Vibin Jacob | Updated Model details |
| 05-12-2025 | 3.5 | Vibin Jacob | Google Search Agent |

# 

# **People** {#people}

| Name | Role | Email |
| ----- | ----- | ----- |
| Manminder Kaur Sardarni | Dev Manager  | manminderkaur.sardarni@motorolasolutions.com |
| Suma Rebbapragada | Product Manager  | lakshmisuma.rebbapragada@motorolasolutions.com  |
| Anjali Kamath | Lead Engineer | anjali.kamath@motorolasolutions.com |
| Sri Chand Jasti | Sr Mgr, IT AIML  | srichand.jasti@motorolasolutions.com  |

**Table of Contents**  
---

**[People	3](#people)**

[**1\. Abstract	5**](#abstract)

[2\. The API Key	5](#the-api-key)

[**3\. Properties of the API Key	7**](#properties-of-the-api-key)

[**4\. API Reference	7**](#api-reference)

[4.1. Chat Endpoint	7](#chat-endpoint)

[4.2. File upload Endpoint	10](#file-upload-endpoint)

[4.3. Get Session Messages Endpoint	11](#get-session-messages/datastore-files-endpoint)

[4.4. Delete Message Endpoint	12](#delete-message-endpoint)

4.5. Initialise Datastore Endpoint                                                                                           13

1. # Abstract {#abstract}

This document is a user guide to interact with the API provided by MSI GenAI, which can be used to programmatically send a prompt and receive a response from AI Models. The endpoint can do session management, so the user needs to pass only the current prompt and session ID (more details below), and the past contexts will be automatically retained. 

The API is exposed at the following link: [https://genai-service.stage.commandcentral.com/app-gateway/api/v2/chat](https://genai-service.stage.commandcentral.com/app-gateway/api/v2/chat)

To interact with the API, an API key is required, which can be obtained from the steps listed below.

**Note: In this v2 release, the endpoint has been changed to /api/v2/chat, which will support a long-lived API key. The former endpoint /chat stands deprecated.**

2. # The API Key {#the-api-key}

Follow these steps to get an API key for the endpoint.

1. Log in to the [MSI GenAI Portal](https://msi-genai.stage.commandcentral.com/login)   
2. After logging in, click on the name in the top right corner of the screen and click on the **Generate API Key**.   
   ![][image1]  
3. Click on the **Copy & Close** button. This automatically copies the token to the clipboard. This token will be visible only once and is not saved or logged anywhere in the backend  
   ![][image2]  
4. ~~The token for the legacy /chat endpoint can be obtained by clicking the **Auth Token (Legacy)** button.~~

**NOTE:** API usage cost is limited to $50 per user/month. The user’s keys will be disabled once they cross the limit in a normal scenario. 

3. # Properties of the API Key {#properties-of-the-api-key}

1) Visible only once.  
2) Valid for 3 months.   
3) The API key CANNOT be used to request content on behalf of other users, however, the owner of the key shall always be logged for audit purposes.  
   API keys will ordinarily not be able to request content on behalf of other users. Reach out to any of the contacts listed if a key has to be revoked.  
4) Upcoming feature to allow customizing the token validity as well as self-service token revocation.

4. # API Reference {#api-reference}

   1. ## Chat Endpoint {#chat-endpoint}

   

* Host: [https://genai-service.stage.commandcentral.com/app-gateway](https://genai-service.stage.commandcentral.com/app-gateway)  
* Endpoint: /api/v2/chat  
* HTTP Type: POSTHeaders:    
  **x-msi-genai-api-key: \<INSERT API KEY HERE\>**  
  Content-Type: application/json  
  X-msi-genai-client (optional): \<some-unique-app-identifier\>

* Request Body:   
  {  
      "userId": "core-id@motorolasolutions.com",  
      "model": "VertexGemini",  
      "prompt": "Who plays the best piano?",  
      "system": "",  
      "sessionId": "c2e07ae5-4d6b-48e6-b035-6a8aefb57321",  
      "datastoreId": "a1c0e7f2-a01a-4ee4-b5fb-e3362bd1f302",  
  "modelConfig": {  
          "temperature": 0.5,  
          "max\_tokens": 800,  
          "top\_p": 1,  
          "frequency\_penalty": 0,  
          "presence\_penalty": 0      
  }  
  }


The different fields of the request body have the following usage.

* userId (optional): The email address of the user in the **CORE ID** format.   
  **Note:** *In case this field is not provided with the payload, all the usages and subsequent costs will be mapped to the user who created the token.*  
* model (mandatory): The model to be used for the AI. The following options are available:   
- Use model: **ChatGPT4o**  to access GPT4 Omni  
- Use model: **ChatGPT4o-mini** to access GPT4o Omni Mini  
- Use model: **ChatGPT-o3-mini**  to access GPT o3 Omni  
- Use model: **Gemini-2\_0-Flash-001**  to access Gemini 2.0 Flash  
- Use model: **Gemini-2\_5-Flash**  to access Gemini 2.5 Flash  
- Use model: **Claude-Sonnet-3\_7** to access Claude-Sonnet 3.7.  
- Use model: **Openai-gpt-4\_1-mini**  to access OpenAI GPT 4.1 Mini.  
- Use model: **Openai-o4-mini**  to access OpenAI o4 Mini.   
- Use model: **Claude-Sonnet-4**  to access Claude Sonnet 4\.  
- Use model: **ChatGPT-o3-pro** to access ChatGPT o3 Pro.  
- Use model: **OpenAI-ChatGPT-4\_1**  to access OpenAI ChatGPT 4.1.  
- Use model: **OpenAI-GPT-4\_1-Nano** to access OpenAI ChatGPT 4.1 Nano.  
- Use model: **ChatGPT-5** to access OpenAI ChatGPT5  
- Use model: **VertexGemini** to access Gemini 2.0 Flash 001  
- Use model: **ChatGPT-5\_1** to access ChatGPT 5.1  
- Use model: **ChatGPT-5\_1-chat** to access ChatGPT 5.1 Chat  
- Use model: **ChatGPT-5\_2-Chat** to access ChatGPT 5.2 Chat  
- Use model: **Gemini-3\_Pro-Preview** to access Gemini 3.1 Pro   
- Use model: **Gemini-3\_1-flash-lite-preview** to access Gemini 3.1 Flash Lite


  **File uploads are currently supported for models VertexGemini, ChatGPT4o, and Claude-Sonnet (Claude-Sonnet supports image files only)**


  


* Supported Files  
  1. 

| Model | Supported Files Extensions |
| :---- | :---- |
| **Gemini-3\_Pro-Preview** | **Image:** JPEG, JPG, PNG or WEBP format.**Video:** MP4, WEBM, MKV or MOV format.**Document:** PDF or TXT format.**Audio:** MP3, MPGA, WAV, WEBM, M4A, OPUS, AAC, FLAC or PCM format. |
| **ChatGPT 4o** | Text, image |
| **ChatGPT 4.1 Mini** | text and vision |
| **ChatGPT 5.2 Chat** |  |
| **ChatGPT o4 Mini** | Text, image |
| **Claude Sonnet 4** |  |

  2. Check model Access and privacy for model accessibility  
* prompt (mandatory): The plain text query.  
* sessionId (optional): All the messages linked to a sessionId are treated as part of the same conversation. **The sessionId should not be passed for the first message.** The AI engine generates a session ID after the first successful prompt, and that value should be used in subsequent API calls.  
* system (optional): Optional system message that can be configured for the model. It will only work with the models that support it. Ignored for non-supported models.  
* datastoreId (optional): Optional DatastoreId that can be passed to refer to the files of a datastore within your chat session.  
* modelConfig (optional): Expert settings to tune the model parameters.  
  If model parameters are not mentioned in the request payload, they will set by default as below:

  temperature: 0.7  
  max\_tokens:(800 Azure OpenAI),(4000 Gemini),(1024 AWS)  
  top\_p: 1.0  
  Top\_k: (NA Azure OpenAI), (32 Gemini), (250 AWS)  
  frequency\_penalty: 0  
  presence\_penalty:0

  

* Response Body:  
  {  
      "status": true,  
      "success": true,  
      "sessionId": String,  
      "sessionTitle": String,  
      "msg": Text,  
      "valid\_response": Boolean,  
      "initialPrompt": Boolean,  
      "args": JSON  
  }


The different fields of the response body have the following usage.

* status: states whether the API call is successful or not  
* success: always true (internal reference key).   
* sessionId: UUID for a session. One can use the same sessionId to maintain a conversation in the same session  
* sessionTitle: Title created for the session  
* msg: Actual response from AI models for users prompt  
* valid\_response: always true (internal reference key).   
* initialPrompt: states whether the user is starting a new session or an old one. If the user sends a message for the first time in a new session, then this will be true; else false.   
* args: Contains metadata about the response

* Error Response Body:

{  
		status: false,   
		msg: String (Error Msg)  
}

The curl command for the request looks like this:

curl \-X POST \-H "x-msi-genai-api-key: \<key-value\>" \-H "Content-Type: application/json" https://genai-service.stage.commandcentral.com/app-gateway/api/v2/chat \-d '{"userId": "String", "model": "String", "prompt": "Text", "sessionId": "String", "datastoreId": "String"}'

Here,

- \-X refers to the HTTP Method we want to use  
- \-H refers to a header, you can use multiple headers by using multiple ‘-H’  
- \-d refers to the data or body of the request

  2. ## File upload Endpoint {#file-upload-endpoint}


* Host: \-5  
* Endpoint: /api/v2/upload/\<SESSION-ID\>?userId=\<CORE-ID\>@motorolasolutions.com  
* HTTP Type: POST  
* Headers:    
  **x-msi-genai-api-key: \<INSERT API KEY HERE\>**  
  Content-Type: multipart/form-data  
  X-msi-genai-client (optional): \<some-unique-app-identifier\>

**Note:**

* The SESSION-ID is mandatory, and hence a file **CANNOT** be the first message in chat history. Obtain a session id by initiating a chat session specified in 4.1   
* The userId is optional and by default is set to the user who created the token.  
* The owner of the SESSION-ID should match the userId

The curl command for the request looks like this:

curl \-X POST \--location 'https://genai-service.stage.commandcentral.com/app-gateway/api/v2/upload/\<SESSION-ID\>' \\  
\--header 'x-msi-genai-api-key: \<key-value\>' \\  
\--form 'file=@"/var/pdf/myfile.pdf"'

3. ## Get Session Messages/Datastore Files Endpoint {#get-session-messages/datastore-files-endpoint}

   

* Host: [https://genai-service.stage.commandcentral.com/app-gateway](https://genai-service.stage.commandcentral.com/app-gateway)  
* Endpoint: /api/v2/getSessionMessages/\<SESSION-ID\>?userId=\<CORE-ID\>@motorolasolutions.com\&page=1\&limit=10  
* HTTP Type: GET  
* Headers:    
  **x-msi-genai-api-key: \<INSERT API KEY HERE\>**  
  X-msi-genai-client (optional): \<some-unique-app-identifier\>

**Note:**

* The SESSION-ID is mandatory. For Datastores, pass the datastore id as SESSION-ID.  
* The userId is optional and by default is set to the user who created the token.  
* The owner of the SESSION-ID should match the userId.  
* The page & limit are optional and meant for pagination for large message sessions. By default, all messages are returned.

The curl command for the request looks like this:

curl \-X GET \--location 'https://genai-service.stage.commandcentral.com/app-gateway/api/v2/getSessionMessages/\<SESSION-ID\>?page=1\&limit=2' \\  
\--header 'x-msi-genai-api-key: \<key-value\>' 

The response structure is 

```
{
    "status": true,
    "TotalSessionLength": "134",
    "data": [
,
        {
            "id": 1,
            "msg": "hi",
            "role": "user",
            "type": "text"
        },
        {
            "id": 2,
            "msg": "Hi! 😊  How can I assist you today? \n",
            "role": "assistant",
            "type": "text"
        }
    ]
}
```

**Note:**

* The status will be true if the data is successfully retrieved; false otherwise  
* The TotalSessionLength gives the total number of messages in the specified session.  
* The data has the list of messages (as per page and limit specified, if any) ordered by increasing order of chronology (most recent message is at the last of the array).   
* Each msg has a unique id which can be used to delete the message from the session.

  4. ## Delete Message Endpoint {#delete-message-endpoint}


* Host: [https://genai-service.stage.commandcentral.com/app-gateway](https://genai-service.stage.commandcentral.com/app-gateway)  
* Endpoint: /api/v2/entry/\<MSG-ID\>?userId=\<CORE-ID\>@motorolasolutions.com  
* HTTP Type: DELETE  
* Headers:    
  **x-msi-genai-api-key: \<INSERT API KEY HERE\>**  
  X-msi-genai-client (optional): \<some-unique-app-identifier\>

**Note:**

* The MSG-ID is mandatory   
* The userId is optional and by default is set to the user who created the token.  
* The owner of the MSG-ID should match the userId


The curl command for the request looks like this:

curl \-X DELETE \--location 'https://genai-service.stage.commandcentral.com/app-gateway/api/v2/entry/\<MSG-ID\>' \\  
\--header 'x-msi-genai-api-key: \<key-value\>' 

5. ##  Initialise Datastore Endpoint

   Host: [https://genai-service.stage.commandcentral.com/app-gateway](https://genai-service.stage.commandcentral.com/app-gateway)

   Endpoint: /api/v2/initDataStore/datastore/\<DATASTORE-NAME\>

   HTTP Type: POST

   Headers:  

   **x-msi-genai-api-key: \<INSERT API KEY HERE\>**

   X-msi-genai-client (optional): \<some-unique-app-identifier\>

**Note:**

* The DATASTORE-NAME is mandatory   
* Datastore ID can be retrieved from the response payload  
* You can upload files to the datastore using the file upload endpoint.


The curl command for the request looks like this:

curl \-X POST \--location 'genai-service.stage.commandcentral.com/app-gateway/api/v2/initDataStore/datastore/\<DATASTORE-NAME\>' \\  
\--header 'x-msi-genai-api-key: \<key-value\>' 

6. ## Get Chat Sessions Endpoint

* Host: [https://genai-service.stage.commandcentral.com/app-gateway](https://genai-service.stage.commandcentral.com/app-gateway)  
* Endpoint: /api/v2/getChatSessions/\<MODEL\>  
* HTTP Type: GET  
* Headers:    
  **x-msi-genai-api-key: \<INSERT API KEY HERE\>**  
  X-msi-genai-client (optional): \<some-unique-app-identifier\>

**Note:**

* The MODEL is mandatory PARAMETER.

The curl command for the request looks like this:

curl \-X GET \--location 'https://genai-service.stage.commandcentral.com/app-gateway/api/v2/getChatSessions/\<MODEL\>' \\  
\--header 'x-msi-genai-api-key: \<key-value\>' 

The response structure is 

```
{
    "status": true,
    "msg": "Session fetched successfully",
    "data": [
,
        {
            "sessionId": "7cads-a9123-1a1ddd",
            "sessionTittle": "hi",
            "sessionchatinstruction": "",
            "total_tokens": 0
        },
        {
            "sessionId": "8bads-a9123-1a1ddd",
            "sessionTittle": "why is sky blue",
            "sessionchatinstruction": "",
            "total_tokens": 0
        },
    ],	
     "user_cost": "0"
}
```

**Note:**

* The status will be true if the data is successfully retrieved; false otherwise  
* The user\_cost will return the monthly usage cost value of the portal.  
* The data has the list of messages (as per page and limit specified, if any) ordered by increasing order of chronology (most recent message is at the last of the array).   
* Each sessionTitle has a unique sessionId 

# 

5. # API Error Codes

| General Error Codes |  |
| ----- | :---- |
| 400 Bad Request | The request was malformed or contained invalid parameters. |
| 401 Unauthorized | The user is not authenticated or lacks permission to perform the requested action. |
| 403 Forbidden | The user is authenticated but lacks the necessary permissions for the requested action. |
| 404 Not Found | The requested resource (model, user, API key, session, etc.) was not found. |
| 405 Method Not Allowed | The requested method (GET, POST, PUT, DELETE) is not allowed for the resource. |
| 409 Conflict | The requested action cannot be completed due to a conflict, such as attempting to create a duplicate resource. |
| 500 Internal Server Error | An unexpected error occurred on the server. |
| 502 Bad Gateway | The server received an invalid response from a downstream server. |
| 503 Service Unavailable | The server is currently unavailable. |

| Error Codes Breakdown |  |
| :---: | ----- |

**fileHandlerService:**

| uploadFile |  |
| :---- | :---- |
| 400 | Invalid Datastore ID. |
| 500 | Error while uploading the file. |
| **deleteMessage** |  |
| 400 |  Invalid request, missing msgId. |
| 401 | Unauthorized. |
| 404 | Message not found. |
| 500 | Internal error while deleting the message. |

**usersHandler:**

| getSessionMessages |  |
| :---- | :---- |
| 400 | Invalid request body, missing sessionId. |
| 401 | Unauthorized access to the session. |
| 500 | Internal error while fetching session messages. |
| **chat** |  |
| 400 | Invalid request, missing model or prompt, or model not found. |
| 500 | Internal error while processing the chat request. |

**Error Response Format:**

All error responses follow a consistent format:  
json  
{  
"status": false,  
"msg": "Error message. Correlation ID: \<transactionId\>"  
}

**Note:**

* \<transactionId\> is a unique identifier for the request.  
* The error field might be present in certain error responses, providing additional details about the error.

6. # Model Access & Privacy

### 

| Model Name (ID) | Friendly Name | Privacy Status |
| :---- | :---- | :---- |
| ChatGPT4o | GPT4 Omni | **Public** |
| ChatGPT4o-mini | GPT4o Omni Mini | **Private** |
| ChatGPT-o3-mini | GPT o3 Omni | **Private** |
| Gemini-2\_0-Flash-001 | Gemini 2.0 Flash | **Private** |
| Gemini-2\_5-Flash | Gemini 2.5 Flash | **Private** |
| Claude-Sonnet-3\_7 | Claude-Sonnet 3.7 | **Private** |
| Openai-gpt-4\_1-mini | OpenAI GPT 4.1 Mini | **Private** |
| Openai-o4-mini | OpenAI o4 Mini | **Public** |
| Claude-Sonnet-4 | Claude Sonnet 4 | **Public** |
| ChatGPT-o3-pro | ChatGPT o3 Pro | **Private** |
| OpenAI-ChatGPT-4\_1 | OpenAI ChatGPT 4.1 | **Private** |
| OpenAI-GPT-4\_1-Nano | OpenAI ChatGPT 4.1 Nano | **Private** |
| ChatGPT-5 | OpenAI ChatGPT5 | **Private** |
| VertexGemini | Gemini 2.0 Flash 001 | **Private** |
| ChatGPT-5\_1 | ChatGPT 5.1 | **Private** |
| ChatGPT-5\_1-chat | ChatGPT 5.1 Chat | **Private** |
| ChatGPT-5\_2-Chat | ChatGPT 5.2 Chat | **Public** |
| Gemini-3\_Pro-Preview | Gemini 3 Pro | **Private** |

[image1]: <data:image/png;base64,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9j0pcF9qSYHLYQtcsG8ocAAAoLIEBe5gClwu2SeybyhwAACgslDgvChwAACgMlHgvChwAACgMlHgvChwAACgMlHgvChwAACgMikF7sLLrzfxJX1/U0yY9KL5YNPmRHbxj26w23340WGZ9fNs/GCTvbzu5juibO78hZn19hQKHAAAqExKgXO2btuWyZpqVd0ae9npptuibP7Cxfby0eGPZ9bP4wqcuP/hRzP372m7XODufXBAJgMAANhjmlDgVq4OS9i7y1aYFavqEvd9+OGH0X31Gzba624dMa1mlln87rLEdu97aFDi9hPPPG+OOP508/vHRkRn/GS7z77wF3vdZXIGbsjo8fb61Fdq1Mdu2hye9Xv6uSmJ52qMXS5wu3u6EgAAQNWEAvfqzDdseZKzaOkC5y5dLh9tTn55hr3+33/4kzn53P8w6+s3mAWL3422O27CJHvZtsO15qquv7Af2brHug4k23XbjH+EGi9w8jjfY6XU1b33vul2+6+j520sChwAAKhMTShwW7ZstaVrxLgJaoFbsnS5GfnEM2bZilU2++HVP7XX17y31mzfsSPartwe89TE6Oyd3DdnXq0te/ESdvWNt9jtLV+52mZS4B56ZKjN3Bk432N37NhpZx465snM11WEAgcAACpTIwrcgapJBa5oSa8PAACwyyhwXk0qcHEUNgAAsFdR4LwocAAAoDJR4LwocAAAoDJR4Lx2ucDx/wMHAAD2Kgqc1y4XOAAAgL2KAudFgQMAAJWJAudFgQMAAJWJAudFgQMAAJWJAudFgQMAAJWJAudFgQMAAJWJAudFgQMAAJWJAudFgQMAAJWJAudFgQMAAJWpoMC5/yrUn6e8lLlvV114+fX2su699xP53PkLM+vuSxQ4AABQmRpZ4Dre0NO073Sj2fjBJtP9rvvMilV15rkpL9v77ntokNm+Y4epmfWm+eHVPzXLVqwyI594xrw4rcZs277drrNly9Zom67Abd22zSxfudquO3vu/KjASS6X769dZ+o3bDQ333mvvb2ufkNmvr2JAgcAACpTIwqcLCtXr7HFy5UvKXCbNm82Q8c8aW9L2ZJFypzc5x7vytfjT/85s81j//38aF0phvECN2T0+MQcrc++2Ex/7a+Z+fYmChwAAKhMjShw7nq6wMmZuM0NDfb2L+75jTn53P+wBW7nzp02+2DT5sRl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>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAnAAAAFMCAYAAABLQ4HoAABBfUlEQVR4Xu3dB5xdZZ3/cfe17MKiSAuQkJBKekN2UdHV1f3roou7rrq2VdfCIqIoCFIEREFAqhTpvfdi6DWAdCI1hBZaIBBCCiSkF57/fJ+Z353nPuecO2fO3Jlz78znvF7vVzL3nn7PnOc7z/Oc535gnfU+6AAAANA8PhC/AAAAgMZGgAMAAGgyBDgAAIAmQ4ADAABoMgQ4AACAJkOAAwAAaDIEOAAAgCZDgAMAAGgyvTvArQsAAEoXl8/ost4V4OILZt31AQBA6aLyOS6/0Wm9I8AR2AAAaCJt5XZcniO35g5wBDcAAJpYWzkel+/oUPMGuJTwtunA4W745E+4cZ/YwU341I4AAKBBqGxWGa2ymhDXdc0Z4KLw9nf/8CE3dOLH3NhPfNEN3uYzbsD4T7p+oz8GAAAahMrmIR/5jBv3yX/3ZbbKbkJccU0c4NrTuy6EEdt9zm025uOJCwYAADQOldVbt5TZKrsTNXFxeY9MzRfgUppNx3ziC4Q3AACahMpsld2pzalxuY9UTR/ghk/e3g2a/OnExQEAABqXym6V4QS4YporwEXh7W/X/QffKXKLcdsnLgwAANC4+o9rfehQZTkhrvOaPsDpyZb4ogAAAI1PZTgBrpimDXD6wP/27wlwAAA0Kx/gWsry6hCXUv4jgQAHAABKQYArrnkCXErzKQEOAIDmlR7gCHF5EOAAAEApCHDFEeAAAEApCHDZ+o+a5LbZZhtv4qhBifebN8DpAyfAAQDQtCoB7u8JcDECHAAAaEgEuGzrb7KF67dFf2+TTTZKvE+AAwAApSDAFUeAAwAApSDAFecD3JBxrW2sWe2s62w52k1se3+bSaNd//j9LBv0d4NGjnMTJ7evf5vJk9yEMSPcFhulzN/mQwOGu5HjJrnJLctNnjzZmzRpohs/dowbNmhztx4BDgCAppce4DZ0G7bkgNHjJ7pJLeW/5YfJLTlgzLCB7kMpuSHuL7Z+/xFuzIT217bZpuPskaaSRyrrsRyztRu4eXuz5vrDxlXeHz9is8R6+m09se398W74pvF2tnQjJtj6x7rBG7S9HqwzzGaW2bovwPUb7saGwS02eaIbsWXcpruR6z/SDrJVe4Cb5CZOnOgmTJjgxgwf4NYjwAEA0NSSAW5j13/rcb7Mt/I/kR8mtOQQCzltqgLc+HHtmSU2eZwb0i8ls6TYuCVAVQW3hIlupOWYTUe48fb6uOFu/ap1beaGj29fbvzwKOBtMdJNSFu2lAC3wSA3cmL7OidPGOtGDB3sBmw13I0YGyTieF3hQWwz2Y0fvbUbvNWWbsDg4W74mAmVADd+/Cg3cMP6B7iv77S7m/bEdLds+XKn6f3333dz5813F14xxU36l/9IzI9sD0573J9DTU/OeC7xfl4/3ed37r0lS/167nngkcT78TzPzXy56r1f7H+oe2fRYv/e0mXL3O+O+lNieQBAOeIAt+GQMb6sbw1wE924kSPc4IFbucEjRrvxk4JcMXa42zDID2GAa80X49zIEcPcgAGD3bBR46sC3eQxQ5K5JWGIGxPkkQljRrohWw10A4aMqM4xE0a6fn7+MKS1hMQwYG66dXu4kyjgbThifOW9scOCiq2MAPehfq0PNnRLgNtwePvOTG7Z0Y2r3t+o6kSPGRwsFxzE+GGbV6933c3c0LEW4Ma7YQPqG+CO/NOZlRAQTwpyTz/7gvvkjt9KLNesvvSdXdwbc+a6latWuWNOPjvxfld8c+c93NvzF1TO38J3F7kf7b5/Yr48uhLg9HnNeG6mf33NmrXusmtvTCwLAChPdYAb4EaMb62sUbep0YM3rM4BVS17k92oQe3vVQU45ZSohm7jIF/kyTFh7pk8enD0/mZuWBjW2l4PM0yYbcLm1VZj3aDKujYKMlhLtgmbVzMCnOmGABem0Elu5Jbx+y02HeQGDxvhhrUYvGWyrTjVuh90W41uD3DDB9YvwO2854E+ZGhSoPnzTXf4wn+fg49yr74227+uEDf1vocSyzYr1UwtWbqsWwLcCWdc4FatXu3Pm6a1a9e6sy66MjFfvWQFOH2O+tw0Kcg1SgDXfihAT/z0lxLvAUBfUhXgBoxw4ye0BbjxW7vN4hywXtRUOnJg6uuq4YqXq8oxQejKFLYIThztBuTpOxfWtI1tr+UbNNby0+RKk2wl4G0wrL2mb/yIqlrFEgLcYDeqUs2Z4yTFNujvBo4Y48ZPbD3QtD5w9Q5wt911XyWkqdAP3/vlbw5377Y1wXWlJqnRKFApWHVHgLPm0wXvvFupietKM2pH0gLccaed61asWOlfmzd/ofv+bvsmlutpexx4mJu/4B2/Tza9+dbb7sQzL3ADxn8yMT8A9HZhgPvwkFG+jPcBLlHr1SaszQqCWt0D3Hqbt2Sj6v53kyeOd2NHtzalbpwa6Pq3P4wwaYwb6F9rz0TjRwyr/L9Sqzd4bGX9iYcfej7ADXdjc88bSXnwYfLkSW7SxLbw1g0B7ivf/5kvRDUtfm+J222/QxLzPPbUDB8QFEgO/eOplde/8I2d3N33P1IJCqvXrHGPT3/G96WzedTsp0nLqy+dQqCCooKTgo7WEW7rkGNOcnPmzvPziOY/88Ir3NBtP+vfj5sUTzvvUt/HS8egY1FfvSum3FzZjibtn/r2ffl7u1YtH05hkNM6rrnhNl9Dp0nNjy++MsvtfsBhiXMTC5tPH3nsqUo4jsOvnRdtV+dF69d2RLVl3/vp3qnHG28vnkcBTvugc6hJx64wFy/T0Wd3yjmX+FpEncOrb7i1alnbd50f1dLG605z3mXX+GWyppvv/IvbfOz2ieUAoDcLA9xmw8dUAtyEoHatSo8FuA+2j6QRZJLQxLEj3BZRU23706ZtLZCDxrTVuk10I7YIa+PGuAEt8w8cbSEx5enUng9wYce/nCfJ26x6P1pOTP9w5OFuakK1pkRNFoLiedJovtdmvxkVw62T+pYpRGg+K+wVBFTjFU8PP/pkJZxddOUUHyTiKey/FYaVmS+/6pYvX+H/b/uuZl4LbvGkgKIaxVoB7mM7fN09NeP51HWoJvJXvz0ycS5CWofWpeM4+ZyLKz/Hzah2Xnwt4MpV0Zacm/X6Gz5kdTbA6Zyov6JN+r+dX5PnswuDqNap86Jlw9efnflSYt1pdtv3kMr6da3tddARPkzve8jRbtHi9yrv/eaI4xPLAkBvFga4jYeObqAauNBGbsPNtnKDh410o8ZNrBoaLdHPP2h6VZ6q7NeEka1NwpUaNwW8QW6ktVhWHoYI9HyAC6oQo06GFZsOcSPHjnNjW4wcukXb60HNXVUHvzbdFOAsYGiKn2CsRbUyCjkKV6qt+sfPfdUdfdJZlSChmifNFwY4hTUV3EeceEZlPquZCoOBQoT65WneRx5/yr+mWrZd9jqoKqwo/Gh51XTddMc97vfHnlLpy/fMCy/65bUerU+TwoLWEe5X3IRq/de0v/c+9NdKX0DbN4W7WqHF1pt2XGEzqs2nSbVlqt1T7dcLL73qX7MA2NkAp+PRZ2KTQqdCazh/3s/Oag/DmjY7Pzr35156dWJf0ig422TB3mz7/75S+cxmv/mW/8w6st3nv5bYBgA0o6o+cAO3bg9wLYEmLWu011hV55V6Bzh70rPfFpunjDu3iRtsNWltNWvt74WhbIwb1ZaHKvsa9HmbMHpMJexN2Lp/Yh9yBbjwhGwzfuvoqdGUpzvijUQ2q1Qhpq0v6ynUMMBFj+DKpkPd6Ak9F+DC18NJtWSqjXl51uv+Z/1rtTOiMKVJNTyf/+8fZgYlbUuTQoMCiMKKQosFF5tvv0OO8cNfWA1WGFYUfMJAoKZPBTaFtLDDftiMq+XD1+L9sv1Xv7Hv/GSvyuvX3ni7fz1uCg2FzdFh0AvXGe9THIQObQmh1qyp2sTOBjibdM6sFlFN1TZvZz67g48+yddwhs2o1r8vPj9Ztpzwz6071DLd/8hjifflnEuursyTZzr9/MsS6wCAZlQV4D60lRtpAU6D9g6N+oT1C8Zaq/UUah0CXHtTaMqoGFXbSz6smRjSpGpfw6dOTRwC2+QJcOsPHls1WN3E8aPd8CFbuS00btuYidUD2eUIcOtsED7I0D4O3BYDh9UYBy5Irf69cX6Zzbcc6oaNHOsmTJrkw1u9A1xWE2qtABcGlazJCvisoBQHOK23o0khKu7vFR+PwpX67OmY4mbQPAHO9itrUjBSqIy3G58z7cPlf77JU/87TQqnNm/W9nXOdO40aZ4iAU41jmrqtZo/7fOvDz3Wz9uZz07hTs2nmhT2dtpj/8q+haGwliEf+UxlveqbGL8vqtXrzESAA9BbxOPA9Rs2NmMcuDFufI2my3oHuHX6hWO3tY4DN6wtFw2vGlcuGvpDKv3eLM/YAw2twqHWklkokCfAxbViMQ2YN9wSY9aGIuv337rqZCdMbglh/au/iWHDoclRjytPoY4b4TYbMrbuAS7PQwxhc5+C1n//8OfurbdbO8lnTRaWsoJKHOAuvfaGcPHUKQ40cYALB61VLZKaajW/mm41dRTgVGNmTZhZU3wcWecpa4rnjdfX1QCnGrPfH3uyf13n1PodWl/Dznx2WodqB7UOhcDrb53qQ6hqCFVTGO9HlgULW588fenV19xmYz5e9Z4eXHj0yRn+fW1HtXSiPoCa1D/QXrNjJMAB6C3iALfOuv3coJaA1JVvYqhLgFsvzzcxtAS7EclgVT0axzZVQ4p40cC+aeHMyxfgZCPXb/BIN3ZicLImT3LjRw1x/TYImllzBjhvoy3dYD3BMal6nbW+j+xDA6q/v0zffTZu5BC3qT6sIa1PqNQzwIk1i6nG6pap91a9p6ZIFbw2KcCpec06wcfNcEbNmQoMWUElDnA2rEfchBrSdmoFOKvF0zdJHHD4cZXX1Zct3JZey9ov9VPTlNVEqGNK+1aKPDVbmmydtn2d84uvuq6yHmu21KQnMzsb4MJzoocgXntjjn/dgl1nPjv9X83RFortYQtrYo2XyxLWrqq5dORHP+9fHzT50+7K626pvHfX/Q9XltG+atL+22t/feJp/xoBDkBvkQxwrd+FuunAEYW/C7VeAU7W33ywGz6m+sEFBbeJ48e4YQM2Scxvwq5pY4bEXxsajpeb8ayA5A9wDW5dsQ+3vl9mH9ZcKdBMufkOX0Crf5lqbsKmSBXGWkZBT6+rQ7ueEFWhr47uNnxFZ/vAhYMJq/ZFPys83nj73b6zfVofuDjAWS2e5tVDDeqcr2+YsCcdwwCn/mWatF9/PLV9mA1tw7anQKF90BhqVjOX1QcubD5Na15UGNNk4TSsrVMNlQJn+BCDzquaFrsS4MSOR5MeJlAwy/vZ2TrCrwVLG1akI/ocbP9sHdOfed7X6tmk0P2Jf/9mZRkCHIC+ID3ASUoOQBUCXJtjTzmnUvMTTwoAVgBbgFO4s9qdeArHHssb4PSaAljaMCKa7JsEaoUVFfrxMWjfn3/xFf//cFtW4xdOOjZtQ0NvxP3nNNX6Oqqs4zT2MIYeFAjnzxpGJO14iwS4sAbVmj7zfnYm/GaJrGb2juzwjR9V/kiIJ4U3PV0aLwMAvR0BrjgCXEBBQJ3v7SlIhRh9mf2p517i/vLgNP+aBThRE93td99feQhC4UthYb/ft3fyzwo2aQFONJCvaoAUlrR9BQb1vbKnSmuFFdHAvuFgwRoK4/jTz/f/D7el9akWzo5V2zv74qv8ezYYsAKHDbeh2imF3Hh7EjafKhjFgxOLPRCQ9nCHBvLVebN9tkGH4+MtEuDEBuXVpLHbdOx5PjujwKfrQFNHw6jUMupj/+aDs/qzqR+emnB1PX3ks19OzAsAfQEBrjgCHEqRFWwbjcKexopTiLVm3XgeAEAxBLjiCHAoRaMHuLSHMtS0nPbQAwCgGAJccQQ4lKLRA9yXvrOLe2XW7EoTsh6AoJ8aANQXAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa44AhwAACgFAa64XhHgAABAcyLAFdMrAlyc6AEAQOMjwBVHgAMAAKUgwBVHgAMAAKUgwBVHgAMAAKUgwBVHgAMAAKUgwBVHgAMAAKUgwBVHgAMAAKUgwBVHgAMAAKUgwBXX5wPcxtvt6Nb5yYnuA/tfAQAAclLZqTI0Llc7gwBXXJ8PcLoA1/3ewe5DX9gZAADkpLJTZWhcrnYGAa64Ph/g9FdEfFECAICOqQyNy9XOIMAVR4AjwAEAUAgBrjwEOAIcAACFEODKQ4AjwAEAUAgBrjwEOAIcAACFEODKQ4AjwAEAUAgBrjwEOAIcAACFEODKQ4AjwAEAUAgBrjwEOAIcAACFEODKQ4AjwAEAUAgBrjwEOAIcAACFEODKQ4AjwAEAUAgBrjwEOAIcAACFEODKQ4AjwAEAUAgBrjwEOAIcAACFEODKQ4AjwAEAUAgBrjwEuB4McBv++y7uc3se6X505JnugDOv9PT/f/3lH9yHv/jjxPwAADQyAlx5CHA9EOAm/GB/d97N97p33lvqsqYFi5a4c266x4373/0SywMA0IgIcOUhwHVjgNv0P3Z1f7ziZrd6zdo4r2VOK1evdn+4+Hq30Y67JNYHAOg7NvnSru7GB59wi5Ysc9vt8tvE+3lp2TseneFOvvYOt0GdW3sIcOUhwHVTgBv93X3cUy+9Huez3NNDM150w761Z2K9AIC+4br7H6uUCfPeXez+6ccHJebpyLY7/8Yva9ORl9yQmKcrCHDlIcB1Q4Cb9KMDqn5hik6z5s53I769V2L9AIDeb/rL1ZUAnQ1xk3c6wL21cFHVOp577c3EfF1BgCsPAa7OAW7IN37pXn7z7apfmHia2/ILddHtD7iL73igw6CnWrwtvrJbYjsAgN5NYS3uO503xE384f5uzoJ3q5Zd+/777lsHn5yYtysIcOUhwNU5wCmY1Zpmzn7L9fvPn1bm3/zLP/M1bbWmE666LbEdAEDv9/FdD/Z94MJJIU5No/G8Rl14Zs9bWLWMwtt3Dz0tMW9XEeDKQ4CrY4DbZqcD3fstvyS1ppOuvT2x3Cl/vjOerWpavnKVG05TKgD0SZ/6+aFuybIVVeVCVohTeJv1VnWlgMqlnY48KzFvPRDgykOAq2OAO/fmv1T90qRN+sUKhwpRNfdrcxfEsyUmPZkab6+oZ2e9Ga8+dTr/lnvdERff4J+MvevxZxPrCeWdL6+vHniCb2rWjeeCW+9LvC87H322e2/Z8ni3/VO/T78y2/30uPMr8+6477HujfnvePp/vC6p9zEU0VP70FPbaRa61jXp3/i9vqSr50H3Fv1O6nczfq+npV3jee4Dpqvnot4+u8cf3NIVKyv3OU1xiFOf6bTwtusfz0usr14IcOUhwNUpwGmQ3oWLl1T94mRN+kvq3qee91S7lmd65tU3Etss6pjLbnKX3PFgxSPPvuS3oX/D13c59tzUm2Cabx58sruwJWgdePZVifeK0PAr2q6q/XXs/b/y88Q8FuDUR+Tqe6ZV9vueJ57zr+vcHn3pjX7ePDfuvMfanep9HrM0wrE2kkYrrMuSdR6Oavk9mvrYM+5ffnFYYpmQ7i0az1JhI36vp6Vd42ol0VAaov/Hy4SyzkWZdvjVUYkyw0Lc0G/u6V56Y27Ve5q6M7wJAa48BLg6BTh9w0JH033TX3CHX3RdlZsfejKeLXMa+7/7JrZbD7VuVGk3wZ6gYVQUzFSTtnjpch8m43kswKWFMt205i96z9fiqTavWQJcT+lLx5pHrd+BviTrPOiPolq/O42oq9d41rko25f3P86tWr2mtVBom95+Z7F/ujSe9jz5ksTy9UaAKw8Brk4BTv0LOprSbn5qQs07KYjEy9dDrRtVV2+CRXz9tyf5G9ITL85yx195i99+WjNqrQAntzz8lFuxarU79IIpBLhIXzrWPGr9DvQlaedBtd/Pvzan5u9OI+rqNZ52LhrF135zYocDxO972uWJ5boDAa48BLg6BTh9r2lH09cOOjGx3Ljv7xfPljn96pRLE8vXQ60bld0E9Re4ApFV36sZ+LKpD1WaNi1MhTfLL+5zjHv0hVcrNxr9q9fibcT01K22edYNd1f6wqU1o3YU4HQ86v+hJp2iAU7bVNOR9j08XvWv0xPFa9au9dtQbaGGhbH375/+gj9Xvz33mqptqAlKfVSsZjDeh7znUT/XOpcdLZP3cxX9/9Qpd/ouAjpWebPlPP7m7Kv9+xa448/o9Oum+u3q33DfHp85y6/re4edVrn2tM2Hn3nJz6/1a+yqX59R+3fzvw44vuoYdQz6rMLmu7zrj38HfnnSxf5c6I+I+LrT8ehzv+KuhxP7JNq+9sPOqZ177a/Nk3X+dR3FDzppyAjNY53YNa+6O8Sff0fXZB7xebCfw6lWHzddt/H7Op7w2tH/42M0e51yiVu6fIU/3vB1NXeqedB+f9VR/8q7H/G18+F6dZ3a8ab9Pov66YX3Ac2v6yM8v/rWAp07TWn3xUag4UD0WadNPRXehABXHgJcnQLc/mdeEf8OZU4KbbZcZwJcRwVaUfFNO2Q3wWUtNzUNd3LVPY+4Gx543D/WrpuHFc5x8NANV2PYqfBSga2+afpXBVlHBYo1n/7oyDP9zwpDac2oHQU4fQWN9l39d4oEOLux6xhUINt+7/6ni9y7S5b6bd86bbq7fOrD7pU5b/v5dNPXPOrDp2aOuCBSoFMBEb9u8p5Hnfusc5lnmbyfq+iY9JqapDUyvM0X9jEMQ5ktp89Rk/611+KwZ9eeCm01AelcqquB9k3neLfjL0gcnyj8apiE8BjV3K6CXIOf2jhZedcf/w7oHL7w+pyq61C0zwp18etG29X2tR+2PR2/9lP7a6E97fzrWtJ+6nW9b/uhzzJtfeFx5rkm84jPg37n1L9Ux2t9TWv1cYsDnLqJ6Dqxa0d0nei4Dzqn9Q+AkAU1nZNwGxbs7Pfipoee8P1jX397QdW1q985/e5pnvj32cQBTrX7ur7t3Gl9Wq+N0Zl2X2wEGh9UNaPxpM9p1Hf2TszfXQhw5SHA1SnA/fCIM+Pfo8ypaID7/uFnJLZbD/FNO2Q3wRdnz60aPNKCiApN/RwHDxVuuqGGhbcKP904w0I+FjafWjixBxriGo9aAU4FpWqJrKAtEuAsuIQFpRXgOrY9WgpNW1bvK5RYiFHho0Io3p7VtsQ1cybvedTPWecyzzJ5P1erAdVn8u1DTqnMt/epl/oCVYNWq1ZRhaAKTxuz8N9+dVRlHKrwHNh2VUjqZ7v2HpwxsyqMKigotNh8MV0LKsA1n72m5bUefWbqpN6Z9af9DqgGWOsKm+91bnVNpdXMiQX3tO1pfxVg9HPW+T/x6tv88vbZ2frufuLZxPq0vN7Pe03G+5om7Tzk+d0xcYBTTZauEwUwm0cP6Figi5cXfSYKeOGDPPq81R1C503jos1ouT4VaMOQF//+xj+bMMBZjXh87uz3Nz4XjULhbdpzL/v9S5u07+FoB92JAFceAlydApx+4fNORQPcp39+aGK79ZB20zZZN0ELGroZhj/bfCowVHDo5hg2HXUkbD611yxEqEYkfHIsLcCp0DrgrKt87YMKaBUgej1PIRQeq2qWFGR0fGEhYcdlxx3SIM5h7YlqAMMaAdsH1TBkPQFXj/OYZ5m8n6v6D6rgjOdTaFA4ULOTHhhRTY1qSW0+Fdhaz19bChkV4FYY65yEhXPWtZe1fybr4RZbTk2TnVl/2nx2HsPrLg6qMV1v4TVg7PyoxiSsAY2PLz7/Wl8cZkTXucKkAkhnrsmOpJ2HPL87Jg5w9jugJ6vTAm8aq23TsvrZakM72n78u5N1jsMAp2tX13Dc/C/afnwuGkFaeNMfB/E4cfoDqidCHAGuPAS4OgW4Db74Y7dgUb5hRIoEOPXZ0Tbi7dZD2k3bZN0E44ImvnnKeTff65dVkFINjoWpWqx5SDds1XwZFX66QalvUrwPaZO2q9ou9ZXRvHkKITtWbU/NUZri+WttU5OOVU1Mmtf6UakJWD9n1SSG8p7HrKbFvMvk/VxrXRta1sJBpemrJTRajZxqNTR+oWqsVKuiAlIBJmwey1p/1v5Jrc8y7/7H68+aT5+dXXcWJLL6L0rWOGjxPsfbN/H+Z60vbZmsKbwmO5J2HuJ9j5cJxQFO50m1jJrsD6Izrr+r5ldBxddS/MeB/N9RZ/vPwvo/hlNnAlzWPJJ2LsqWFt5US6xvWPjM7ocnxonTcXZ3iCPAlYcAV6cAJ7pJ5pmKBDh1zo23Vy+1blRZN7i4oEkLHqKO1lqv5tPNRUEi/kvXWPNp1qSCKAw/tk2FWzXp6WdRzURcQOQphOxYNWk/bpv2tL85qr9NvE0VSra9mI2VZbVUWpeOLasvXyjvebTaxaxz2dEyeT/XWtdGGOD0s2osrLlM/dJUCFszngpbrVs1ReE2s9aftX9S67PMu//x+juaTwHUAnnaPpmswBXvc7x9E+9/1vrSlslzTXYk7TzE+x4vE4oDnOh6U/9d9SNV+NXvlK4D9duLlzc633Yt6Y8Baz7Ve1Yjb/0fta9al2qLw88n6xw3a4CrFd5sns/vdWTlgRibdKwTfrB/Yn31QoArDwGujgFOgymqkMyafnZ8+zcDxH5+woXx7JVJTRDdNQac1LpRZd3g4oImK3iEVBsW16KF1Cylm3Jav6e05izbZp6CJU8hZMeq2jcVCKolenXOvKrO7rYftZpBQ2oK1jFdeueDib59afKeRz1RWetcxuJl8n6uegBE81mTpImbUPWa1qmCdsp9j/pjtSYwfZ4KrgrfKlzC5sesay9r/2zbqskLtx0vV48mVLG+UKoNuv6Bx2v2XxRtV+vVeQtfL9qEqvXlbULNe03WknYe8vzumLQAF1MXB4Uzq5lOY82o6uun6yystbVzFz8IFP/uZJ3jMMBZE6p9LuF8jdSEmie8GR2XjjucFHi7K8QR4MpDgKtjgBM9tZg1xY/9h/Re1pS3+aOotJu2yboJxgVNfPNUIa3gGXYyFxuXLd6OqPk07vBsrKN2GFq6K8CFx2qv2dObth96TaPO23x6XR3X41BlhateV5AL+/alyXseFYqyzmWeZdKONdy+fa4dPcQQBmqrQVUHda1D29DrCh+aV10M4g71Wdde1v6ZWg8xhP0O864/az6xflz2xGjWE5jS0UMMVoMcb9/E57/W+rR8+BBDR9ekQvwP/nCGZ10LYmnnIc/vjgkDnDWFxteOXSf2oEwaW1bXnpriw7Bm5+7P9z5atYzuvTrHnQlw9hCDrk9d0zZPIz3EsPmXf5YIbx19Mf1XDjy+x0IcAa48BLg6B7gh3/ilr7VJm66996+Jb2IwuhmlTXrSasBXs2ts6iHtpm2yboJxQRMHD6u9Cod50FeH6fH8tELAburW7yV+X+ypQCsEeyLAqRBUeNPrFkhsyAYbL0pDNuhz0k01HG7EqKZBkzWlxtsN5T2P2nbWucyzTNqxhtu3z1XyDCNitL34WO3ca4o7i2dde1n7Zzo7jEhH67eHNdQMucsx51TtozWdakqrHQ51dhiR+Pji819rfWnDiNS6Jm3dFrDifZe082VhSs3w+vw7M4yIjZln144N0aH9TxucO6R5NcW1ngqg+kNA509Dwtixar3aRzun9jS2gouacC20hgEu3EfNq2vb9lHHrM89vnZ60ub/tZt74OmZ/jzYpGshzxfT6/dPxxVO6m4SduGpBwJceQhwdQ5wMnmnA/wNpquThsHI+ku5ntJu2iZvQRMHD1GtogYctT4Zummr83G8DTnlz3f6963ZLY01Q1mtT08EOFHzqQpHhSIruDRoqgKDdaLWjV4Di6Z9XtYMWavJyOQ9j9p21rnMs0zWscafq6jwVx9MFcIqPCQcyDekwlChQaEqfN3GhIsDUNa1l7V/IRvIV8FLk441ayDfjtavIKQmZhV48TWi47cm2zxN1tq+9sM6lOvcZw3kGx9f2vnXvsUD+epaikNUR9dk0QAnR15yg//d06SgkzYGnsQBTo6/6lYfHOzayTvAsDWjptV6KpDZOu386trWZ2fnVOtX/1Wd53Cf4gAn4WDDuga0PtXI6Vjjc9GT9E004ZQ3vJn/+f2p/vcxnE6bUj2wdlcR4MpDgOuGACf6K0djFRWdHmu5gWz9P79KrBfNR7WrYQ0emst3Dj3VD+oa1x4C3U0h1qbOhjfzv4efXumbrX+z/oAtigBXHgJcNwU4UfW3/rKL/wKqNamg19ALm3xp18T60DxUW6hmPTVnqcYkHsMOjU2flQYDtm800O+l9asDepJqfXUN5h2MOY2WVTeKrJrTriDAlYcA140BzujpVA2oaU0QaZM6eKtvSXeP2YOeobGu1BSjv3jVr6pWcycaj75JQoWmJjWFxt8PC6AVAa48BLgeCHBmox13cV/Y+2hfDX7AmVd6+otI/Ts+3E2D9AIA0F0IcOUhwPVggAMAoDchwJWHAEeAAwCgEAJceQhwBDgAAAohwJWHAEeAAwCgEAJceQhwBDgAAAohwJWHAEeAAwCgEAJceQhwBDgAAAohwJWHAEeAAwCgEAJceQhwfTjA6Uuyn3hxVuV78sIv0O4O+gJq+6Jt/asvh0/7QvRmo+8W1Jdjx1+QXU9fPfAEf77ss4q/BL23y/qSdQDlIsCVhwDXhwPc6ddN9d/T+tIbc/33dh5z2U2Jeerl6Etv9N8Jqu/j07b0r37W101dfMcDifkb1VEtxzH1sWfcv/zisMprPRHgbnzwCR/eHp85y5+/A8++KjFPI9DXTd3wwOP+a+Hi97qCAAc0JgJceQhwfTjA9VShqNqjuQsXuUVLllW9rsCoLwnvzuBTb/c88Vxif3siwKnGTefqiItvSLzXSP5z/+PcnAXv1r2GsKeuVQCdQ4ArDwGOANfthaJqi5atXOVrj8LXLfi8t2y52/noxv+yd9UuPf/anERQI8C12/WP57kly1YQ4IA+ggBXHgJcHwxwVhjGkxW6CiqnTrnTLVy8xDfbyZst4STsrxaGlvNuvte9895Sv47OFLC/Pfca34w66635VU2SIfXLU8A77opb/D5oX7TMHY/O8Mtc/8DjbumKlf71txYu8v3swuX/64Dj3aMvvFrpe6dl1QT62T3+kLoNHY/WpaZdLffFfY7x86SdMwue4bkIz5u2Ga5D9P9wf9LmCekzSZu0PwpzCnVqjtb50P/DMJzn2O24Lpv6kHvk2Zf8ccvM2W+5/zvqbHf4Rde5t99Z7I9H51nz6fqI99POYzyF/Srz7I/WrW0oBNo8OjY1s2sKry9b34pVq/17tt8/Pe58//4Vdz3sXzvrhrsT+6o/JrSNX550ceI9APkR4MpDgOuDAW6XY8/1/ahUYGvSv2G/KhWWKvjmL3rPXXf/Y75Pk5o/VZiqL5vmsdCimjWFBoWIy6c+7PY65ZLE9mJfO+hEX6hqnVr+yEuya5UUALQvi5cu9wW57YsChULj628vcFfd84i7b/oLPsAoPP3gD2f4ZdV0O3veQh8YrO/Y06/M9stOf/l1/xBHvA07jhden+Pn0/zb7HSgP2dX3zPNb1P0f/XzUviwc7Fq9Rr/3q3Tpvt1vDLnbb8OhQwFE63nqZder9of/WthMS0Y6TPRfKr503IKdPpZ+2MBTvs9793F/rO66Lb7/f7kPXYLcEuXr/Dvh/utgGPHo3Xr3Gpf1Xcy3k9Rk7j2T9vU/ob9KvPuzwW33ue3oWtK29Vnq89YxxcGOJ13PdQRru+hGS/6n9Vcr+0pnOkY9KBOeG5/dOSZ/rji1wF0HgGuPAS4PhjgTFqzlPVXU63Ltw85pfL63qde6gv5l99829d8WWhRgOjMww8qmG3SdjoKfApXCkYnXn1b5TWrubOC2l5XeNBDGRYwVAOjnxW2bB4V2A/OmOlDwsnX3pG5DQWKF2fP9cds+5jVVGqva16dJ3tdIWPW3Pk+cCo0iP6voBHuj37W8t877LTK67G0JlQLcKqZDM+D5D12uwb0uoUZ22+dkz9ecXNie3FTeMjmiZtQ8+yPrivVxuoc7fGniyrz2f6E16rOs0LdlPserdpOeJ60foU0BVwFXptHfzwo6KXVzAHoHAJceQhwBLiqAHfoBVN8k1RcAKswfObVN3yNhvo5WWhRh3V1XI/XnUW1Ywpgql1RmFNhqybYeD5jzZthHzn9X6/Fw57Ex6ManrjwFgsZeiAhaxsSh6aOAlz8erxuBTTVYimkqPkvnK8j8b5IfByhvMcenzOTdk6yznva+uPrJ8/+WP85XWdxzZiewk3bz5jeD8+TwrwCogKkfrbrWH+gfP23JyWWB9A5BLjyEOAIcFWFYtprJgwRtUJLXrv/6SL37pKl3m7HX5B4XzoTJMJ9r7V/8fJp25A4NGWtM+v1tHUrrGqdajpUiFCzcNax19oXyQpLtfYnPvaszzve77Rl06TtU979SVvWpO3nSdfe7pv5dS7DKTxPVqOsJnE1YVuz6v3TX0hsA0DnEeDKQ4AjwHV7gFPTmArqtAcVVPMSNnvGOhMkmiHAiR5Y0D7qPXsAQ0EurnWqtS+SFXhq7U987Fmfd9p+x8umSdunvPuTtqyJ91NNu2ri1bKqzVUTrNZ1zV/+mjhPWp81hasmTjXM4fsAiiPAlYcAR4CrKrw1UK0KwLhZLqsJNa1Qjp1w1W2VDvjxe+r/1R0BTvurjvS2v+F8FhR6ugk1fN2M/u4+7uFnXurwich4XyQr8HTm2NOuAUnb76zznrb+cJ/y7o81oWreOMzGTahav/pBqjk+nE/vx+fJ+kze8vBTftBq9acLn3wFUBwBrjwEOAJcVeHd0UMM1hRVK7TE7Kk/NZWGr1sTaq3w0pkgER9PrY7zYQf9tG1IHJqyjjnr9XjdCrLabrg/oictVSuk/ofh67X2RdLCksl77PE5M2nnJOu8h7L2Kc/+2EMMWQ+DxAFOtZf7nnZ5ZT49eKI/MuLzZOvVE8/aloKcvafPS8cVf24A8iHAlYcAR4BLFN6dGUYkLbSksXWqpqUzX6XVmSARH0/eoSvStiFxaFJwVQ2OgoPOSzyMSNq5CNedNvSFnQcNlREvW2tfJCssSd5jj89Z2n7ba1nnPaRmSgUwbbvIMCL20IGuN113NoyIzruCvu2nzaeHQvRZiP7w0HzxeRIFSD+WXfBUsejcaUo7hwA6RoArDwGOAJcovFUzogFprYO41BrIt1bwCGl5FbqaVOOk2ryOvsy+M0Ei7XjiwV7TBo9N24akhSaNWaenKTXZ8CC1zkW8bvV/07h72g9NCjQKMho0N1wulrYvtQKc5Dn2tHMm8X5L1nkP6dq5bdrTlcF6dWyd2R/Rwwk2GLKNkacaOZ1v209tRwP+KkxrUrjTz/p8tI9xgLPmWatBttcJcEDXEODKQ4DrwwEO6CtO+fOdvvlUAwXH7wEojgBXHgIcAQ7olb558MnuwpbAZl0A4oGfAXQdAa48BDgCHNAr/e68a2t+Ty6AriPAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlYcAR4ADAKAQAlx5CHAEOAAACiHAlafPB7h1fnKiW/d7BycuSgAAkE1lp8rQuFztDAJccX0+wG283Y7+AtRfEQAAIB+VnSpD43K1MwhwxfX5AAcAAMpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOAAcAAEpBgCuOANfLfeADHwCAPiu+J6KxEOCKI8D1cvHNDAD6kvieiMZCgCuOANfLxTczAOhL4nsiGgsBrjgCXC8X38wAoC+J74loLAS44ghwvVx8MwOAviS+J6KxEOCKI8D1cvHNDAD6kvieiMZCgCuOANfLxTczAOhL4nsiGgsBrjgCXC8X38wAoC+J74loLAS44ghwLe554BGn6aIrp/ifv/L9n7k333rb0//j+eWYk892K1et8svG7+XxyR2/5Z6a8bx7//33/bafm/lyYp6zLrrSrV27tuo9/f+9JUvdT/f5nTvu9PP8Pmg9Q7f9bGL5fQ4+KnEzA4C+JL4vorEQ4IojwLU4+OiT3OV/vsnttt8h/ueeCHAWzl6e9brftsJYPI9tIwxwmu/8y691X/jGTj60PT79GT+P5g2X1XsPP/pk4mYGAH1JfF9FYyHAFUeAS9ETAU61fWGtX5q0ABf79aHHuqXLlrmZL7/qPrbD1yuv//7Yk93y5SsSNzMA6EvieyYaCwGuOAJcikYLcB1tQ++vWr3anXLOJf7nsGYuvpkBQF8S3y/RWAhwxRHgRifDVNEAZ/3TDjj8OB+g1qxZ6/u4zZk7z/3miOOrthVPaSFtv0OO8bVr4Xv6v/WBs9d22esg986ixW7W62/4ptVDjz3FrVix0u9DfDMDgL4kvq+isRDgiiPAja5vgFu2fLl77Y05vknzqutvcQ888pifT6FLgUz97NTnbdoT0/029a9+Vj+8eBsKaVquowAnt0y9161es8b3j3vm+Rd98FPzanwzA4C+JL6vorEQ4IojwI3ODnB5pjjAaVLNV/hU6KXX3uAfWHhw2uOZ20zTmQD3/d32dfPmL/RhUTV/tkx8MwOAviS+r6KxEOCKI8CNToYpC3Bz5833tVgKS7FzLrk6tQZOtWAnn3Nx1fq/85O9fLh6bfab7vP//cPUbaaxABfOkxXgxILiwncXuZ33PNC/Ft/MAKAvie+TaCwEuOIIcKOTYaorTahp4UpPh2q4kHB98TbT2H7kDXAW+MKnVuObGQD0JfF9Eo2FAFccAW50Mkx1JcAtWbrM/WL/Q6vmVa2bat8IcADQs+L7JBoLAa44AtzoZJjqSoCr1YSqWjgbqy3eZl4EOADIL75PorEQ4IojwI1OhqmuBDhNz858yX9Vlr1+5XW3+L5pt911X+Y2RctoSJAf/Hw//xCEau4Uyr69y56VeQhwAJBffJ9EYyHAFUeAG50MU10JcDaMyKuvzfbDiDzy2FO+Vk7jtIVNq/E2w3Xadm2eMJAR4AAgv/g+icZCgCuOADc6Gaa6EuAUoOKBfDXA7t6/O7LmNsN1EuAAoD7i+yQaCwGuOALc6NbhN+IwVUTWU6hlim9mANCXxPdENBYCXHF9OsDpWxGuvfF2t2DhO27lylXuiBPPSMzTGQQ4AGgs8T0RjYUAV1yfDnD62ik1caqPmpom7QnRoghwANBY4nsiGgsBrrg+HeD6gvhmBgB9SXxPRGMhwBVHgOvl4psZAPQl8T0RjYUAVxwBrpeLb2YA0JfE90Q0FgJccQS4Xi6+mQFAXxLfE9FYCHDFEeB6ufhmBgB9SXxPRGMhwBVHgOvl4psZAPQl8T0RjYUAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgAAlIIAVxwBDgBQw0ddv1FARNdF4lrpPAJccQQ4AECktZDedNR2rUbG/gl9TnQNtF0bXQ1zBLjiCHAAgEBbeGsLaptsva3bZIR8BGizrb8uLNh1JcQR4IojwAEA2lh4aymY//Hzrv8uR7gtf3MhkErXh64TXS9FQxwBrjgCHACgVVvNm2pXVDhvuO+57m92P9194BdANV0Xuj50nbTWxllNXMp1VQMBrjgCHADAa+3v9k9u4+Hb+BoWwhtq0fWh60TXi6+1VZ+4lOuqFgJccQQ4AIBT85cKYNWmbDxski+Y4wIbiPkA13K9+Fo4H+A6VwtHgCuOAAcAcD7Aqfl0xLZuo6ETCXDIRdeJrhddN74ZlQDXYwhwAID2/m8jPtJSIE8gwCGX1gA3wV83RfrBEeCKI8ABACpPn/oAN4QAh3x8gBtiAc6eRk25vjIQ4IojwAEACHAohABXHgIcAIAAh0IIcOUhwAEACHAohABXHgIcAIAAh0IIcOUhwAEACHAohABXHgIcAIAAh0IIcOUhwAEACHAohABXHgIcAKBbA9y+1z3sXp6/2K1eu9Zp0r9PvbHA7XTJPYl5G8Wggy5y1z71ilu6crXf5/dWrHI3PD3LjTv8isS8aT7+x2vdjTNmuUXLV7r333fewqUr3IXTXvDrtvkOueVRt3LNWnfH87MT62gGBLjyEOAAAN0W4M57+Hkf2BSE7p75prvgkRf8v8tXrXHLVq12v28JMPEyjeC66a+6tS2p6+k5C/0+P/r6PH8ctz83222w9zmJ+UNfPO0m9+qC93xo07+XPfai99o77/l1PjF7fiUIEuAIcEUR4AAA3RLgfnH1/W5JS3B7Z9lK95PL761677DbHvMhbva7S9y/nnR9YtkyfeqEKe71d5a4NxctdZ8/+Ub/mkLbg6/MdQuWrnBfP+f2xDJG89330hwf3uKwp9D25BsLfIhTsNVrBDgCXFEEOABAtwQ4hRIFmUv+OjPxnoLNY6/P82HmpL88nXi/TKpBe2vxsqoAJ/e//JZ7tyWM/s/5dyaWMT+4+C7fbKrltZ74/V9f/7APrq8sWOy2O+YaAhwBrjACHACg7gFO4UQhRTVwu115X+J9UdhR8+Qe1zxQeU395d54d2ml35hqvE68Z3pVTdaMOQvd4hWr/HLTZr3t1qx938+r5fa89kE/z0+vuNf3W1NIjJs8FRi1TFqwlFGHXuaem/uOn+fsB5+rLKPglba+eN0KpQp78Xu27uPvnu5OuXeG2/aoqzMDnPrQqQZP27R+gzrWHU5pD5Ry3F1P+XMUni+9Fs6jvobPz323cp7UF+/8R56veRx5EeDKQ4ADANQ9wKmWSrVVcS1WLdasqnB204zXfL+xuYuX+eChwGHzKcCpT536lyloXdwSxO558U0fhLSsAqOFMIWVsMbMav7i12PHTn3SrWpb34vzFlXCUdwUHLv12dd94LIm0o6kBThratU2n3mr9fgUCBXi1LRrNXu/u/mv/nzNW7LcXfPkK572UX0L92sJwppH+6smbDunWtdL8xf5dYXntCgCXHkIcACAuge471041YeGvH3cLHCpxk595+z1r5x1qw9x81tCiv6v1xTgNKlGKqxFUmhS7ddfXpzjfz71vhlVtWii0Kbw1lFNmt7TwxY2KfR894KpifliCmKaznrw2cR7adIC3BG3P+7Do/rShft4+WMv+eNTUNPPCotxDec+Ux6qBDoLqwrSu17RHjwVEPVwRkf9+fIgwJWHAAcAKD3AWY2dgkUcrBRuFHIOvfUx/7MCnGqQjpn6ZNV8/3HGLT7sWf8yhROFFAVDBUTNozCncBQvG1LAUfOltiFxDeBBN01zK1av8QEqXrYeAU7r1Wt6L5zX+tc9+9Y7/hxNmf6qP5ZzHnoucc7Ejl/nK35PYTdtG51FgCsPAQ4AUPcA19km1LQgYxSGwlBkfeAUEsP5FNJmzltUFRpVG6e+cOoTZ7V8WQ8Y2Do0zIeaL7XsN8693TdbqqlSTbyaRyFQ+6qasnj5ejShZh2fjknHZsenY3jh7Xf99rR/Wu7ke2dUhiixEJ016RhPv/+ZxD51BgGuPAQ4AEDdA1yehxjUP8seYkgLMiYtwCmU/eiSu1O3GQY4W6/609mDDWnbMNZ8qWBkQcj6kYn2Ve9pYGI9aBAv39FDDFrnn1rmqfUQQ94Ap9dU86YHN1RjqGCq2kLrq2cBTvur/6fROYv3sTMIcOUhwAEA6h7gRN9c0NEwItYUqjCmcGXNg+G8nW1CVS2cNZkqZOlhBwU7fbOCaqo0lEe8P8bCYhzyLMTpAQGFUj1AEC8r1scuq5bPhhGxfUwLcFNfeKPqeE3chBqvW/ae8pDfP9UeWhNqeD7qjQBXHgIcAKBbAtzOl95TqbmKn96MB/Lt6CGGMBDZQwzqLxd+tdVF02b62i8Fx3Bb1ldM4UthLq3mzFgNWlgDJwpM+nYGTR01CyuMKbgWHci3o4cYFIituVjnxh7uEAux+qoyC8la/+G3tTf36nWt25qW4/3vDAJceQhwAIBuCXBy5gPP+gChsGZfpWVfSxX2K5Miw4jo6dBwmA3VfsVNq9Z0qknrjPcxpJCoPm8KYPrqK82vJzq1H1q/DcERh7N4HQqA8Tr0dKgmhTgLh9ZXUAFVTaH6ntS8w4gobOrc2FOn9nVdms+evLWaQ51X9c/TurROBcFax5AXAa48BDgAQLcFOPn5Vff7QKNgoUn/Zn2ZvQ3ka4POdmYgX/VLC4fLMFpWzY55a5w0XIj2L9xf1fbt1RKwtC57QlU1flkBSCHsqide7vDL7K1mTyE37PdmA/kqqNo+pA3ke/SdT7o5i9oHPk4bpFfnOTwenYdLH32xaj+KIsCVhwAHAOjWAFdvWZ38s3zt7Nvc2+8tTx2iBF1DgCsPAQ4A0OsCnH1llX3zQNawH+gaAlx5CHAAgF4X4D51whQf3DSpGbJWcyeKI8CVhwAHAGiqAIfGQYArDwEOAECAQyEEuPIQ4AAABDgUQoArDwEOAECAQyEEuPIQ4AAABDgUQoArDwEOAECAQyEEuPIQ4AAABDgUQoArDwEOAECAQyEEuPIQ4AAABDgUQoArDwEOANAW4LZrDXBDWwPc3+yeLLABo+vDB7ihFuC2I8D1IAIcAKCFBbhtWwrkiW6LHx/uPvirsxOFNmB0feg60fWi68YHuNEEuJ5CgAMAOB/gRrUEuK23dRsPm+Q2mvhpt8XOh/kaFiCNrg9dJ7pedN3o+iHA9RwCHACglTWj+hA32fdt2nCrse7Dg8a0GO0+PHAU+jpdBy3Xg64LXR+6Tnx4K9B8KgS44ghwAIA2H21/mEEhbvg2rbVxQyf6fk4qsNHH6TpouR50Xej6aA1v9vACAa4nEeAAAAELcdtVgpz6N6mTOtBq20pwa69563x4EwJccQQ4AECkLcSN2q6VD3MhFdzoW6JroO3a6Ep4EwJccQQ4AEANrWEOqNKF0BYiwBVHgAMAAKUgwBVHgAMAAKUgwBVHgAMAAKUgwBVHgAMAAKUgwBXXvAFuXQIcAADNrBLg1iXAdVbTB7hxn9jB9R/3icRFAQAAGpfKbpXhBLhimj7ADZ+8vRs0+dOJCwMAADQuld0qwwlwxTRPgJMgxFmA23TgMDfq4zskLgwAANC4Rm+/gy/DkwEupfxHQtMHuL/9+/Xc0IkfdSO2+1zi4gAAAI1HZbbKbpXhBLhimjbAhSFunfXW9xeCauK2nPDPbvOx2ycuFgAAUJ4txm3vy2iV1SqzVXYnwxsBLq9eEeCsJm7TLYe54ZO2950i9WQLAABoDCqbVUarrK7UvBHgCmuuACeZIU4XQ5u/k3UBAEDDCMrp1KZTwltn9I4AlxbiKkGOMAcAQDnayuG4fG4LbwS44povwEnNEJcS5AAAQANobzYlvHVNEwc4CT/4tCAHAAAaRmpwawtvBLhOac4AJxkhrirIEegAAChPVB7H5TXhrbjmDXBSI8QBAIBG11aOx+U7OtTcAc4Q5AAAaCJt5XZcniO33hHgTCXImfiCAQAAPS8qn+PyG53WuwJcLL5gAABAz4vLZ3RZ7w5wAAAAvRABDgAAoMkQ4AAAAJoMAQ4AAKDJEOAAAACaDAEOAACgyRDgAAAAmgwBDgAAoMkQ4AAAAJoMAQ4AAKDJEOAAAACaDAEOAACgyRDgAAAAmgwBDgAAoMkQ4AAAAJoMAQ4AAKDJEOAAAACaDAEOAACgyRDgAAAAmgwBDgAAoMkQ4AAAAJoMAQ4AAKDJEOAAAACaDAEOAACgyRDgAAAAmgwBDgAAoMkQ4AAAAJrM/weBkigiRuvTtQAAAABJRU5ErkJggg==>