from locust import HttpUser, TaskSet, SequentialTaskSet, task,  between
import uuid
import os
import signal
import sys
import json
import datetime
from HyperswitchSdk import HyperswitchSdk, PaymentIntent, PaymentAttempt

config_set = {
    'version': 1,
    'base': uuid.uuid4().hex,
    'pi': [],
    'pa': [],
}


class PaymentsBehaviour(SequentialTaskSet):
    payment_id = None
    attempt_version = [None] * 1
    session = None

    def gen(self):
        self.session = HyperswitchSdk(self.client)
        config_set['version'] += 1
        return (config_set['base'] + str(config_set['version']))

    @task(1)
    def intent_create(self):
        payment_id = self.gen()
        payment_intent = PaymentIntent(payment_id)

        try:
            response = self.session.create_payment_intent(payment_intent)
            self.payment_id = response["pi"]
            config_set['pi'].append(response["pi"])
            print(self.session.cell_id, str(datetime.datetime.now()))
        except Exception as e:
            print(str(e))

    @task(1)
    def pay(self):
        for i in range(1):
            payment_attempt = PaymentAttempt(
                self.payment_id + 'version' + str(i))
            response = self.session.create_payment_attempt(payment_attempt)
            self.attempt_version[i] = response["pa"]
            config_set['pa'].append(
                self.payment_id + ',' + self.attempt_version[i] + '\n')

    @task(1)
    def update_attempt(self):
        for i in range(len(self.attempt_version)):
            if self.attempt_version[i] is not None:
                attempt = PaymentAttempt(self.attempt_version[i])
                response = self.session.update_attempt(attempt)

    @task(1)
    def update_intent(self):
        payment_intent = PaymentIntent(self.payment_id)
        response = self.session.update_intent(payment_intent)

    # @task(1)
    # def retrieve_attempt(self):
    #     response = self.client.get('/retrieve/payment_attempt/' + self.payment_id,
    #                                name="RetrieveAllAttempt", headers={'x-region': self.cell})

    @task(1)
    def retrieve_intent(self):
        response = self.session.retrieve_intent(self.payment_id)


def make():
    return type("MyClass", (PaymentsBehaviour,), {})


class WebsiteUser(HttpUser):
    MyClass = make()
    tasks = [MyClass]


def sig_term_handler(_signo, _stack_frame):
    print("waht tht fook")
    f1 = open('intent.out', 'a')
    f2 = open('attempt.out', 'a')
    f1.write('\n'.join(config_set['pi']))
    f2.write(''.join(config_set['pa']))
    sys.exit(0)


signal.signal(signal.SIGTERM, sig_term_handler)
signal.signal(signal.SIGINT, sig_term_handler)
