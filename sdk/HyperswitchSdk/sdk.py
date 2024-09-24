import requests


class PaymentIntent():
    def __init__(self, id: str):
        self.id = id


class PaymentAttempt():
    def __init__(self, intent_id: str, attempt_id: str, version: str):
        self.intent_id = intent_id
        self.attempt_id = attempt_id
        self.version = version


def parse_id(id: str):
    cell = id.split("_", 1)
    return cell[0]


class HyperswitchSdk:
    client = None
    base_url = ""

    def __init__(self, base_url: str):
        self.client = requests.Session()
        self.base_url = base_url
        self.cell_id = None

    def create_payment_intent(self, intent: PaymentIntent):
        uri = "/create/" + intent.id
        cell_id = parse_id(intent.id)

        if self.client is not None:
            try:
                headers = {}
                if self.cell_id is not None:
                    headers = {"x-region": cell_id}

                self.client.get(url=self.base_url+uri, headers=headers)
            except Exception as e:
                raise e
            finally:
                if self.cell_id is None:
                    self.cell_id = cell_id

        else:
            raise Exception("Client is not set up yet")

    def create_payment_attempt(self, attempt: PaymentAttempt):
        uri = "/pay/" + attempt.attempt_id

        if self.cell_id is None:
            raise Exception("Cannot make this request without cell id")

        headers = {"x-region": self.cell_id}

        try:
            self.client.get(url=self.base_url+uri, headers=headers)
        except Exception as e:
            raise e

    def update_attempt(self, attempt: PaymentAttempt):
        uri = "/update_attempt/pay/" + attempt.version + attempt.intent_id

        if self.cell_id is None:
            raise Exception("This cannot be called without cell_id")

        headers = {"x-region": self.cell_id}

        try:
            self.client.get(url=self.base_url + uri, headers=headers)
        except Exception as e:
            raise e

    def update_intent(self, intent: PaymentIntent):
        uri = "/update_intent/" + intent.id

        if self.cell_id is None:
            raise Exception("This cannot be called without cell_id")

        headers = {"x-region": self.cell_id}

        try:
            self.client.get(url=self.base_url + uri, headers=headers)
        except Exception as e:
            raise e

    def retrieve_intent(self, payment_id: str):
        uri = "/retrieve/payment_intent/" + payment_id

        if self.cell_id is None:
            raise Exception("This cannot be called without cell_id")

        headers = {"x-region": self.cell_id}
        try:
            self.client.get(url=self.base_url+uri, headers=headers)
        except Exception as e:
            raise e
