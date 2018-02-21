package edu.rpi.aris.rules;

import edu.rpi.aris.proof.Expression;
import edu.rpi.aris.proof.Operator;
import edu.rpi.aris.proof.Premise;

import java.util.ArrayList;

public class DisjunctiveSyllogism extends Rule {

    DisjunctiveSyllogism() {
    }

    @Override
    public String getName() {
        return "Disjunctive Syllogism (" + getSimpleName() + ")";
    }

    @Override
    public String getSimpleName() {
        return "∨ Elim";
    }

    @Override
    public Type[] getRuleType() {
        return new Type[] {Type.INFERENCE, Type.ELIM};
    }

    @Override
    public boolean canAutoFill() {
        return true;
    }

    @Override
    public ArrayList<String> getAutoFill(Premise[] premises) {
        //TODO
        return null;
    }

    @Override
    protected int requiredPremises() {
        return 1;
    }

    @Override
    public boolean canGeneralizePremises() {
        return false;
    }

    @Override
    protected int subProofPremises() {
        return 0;
    }

    @Override
    protected String verifyClaim(Expression conclusion, Premise[] premises) {
        Expression p1 = premises[0].getPremise();
        Expression p2 = premises[1].getPremise();
        int found = -1;
        for (int i = 0; i <= premises.length; ++i) {
            Expression e = premises[i].getPremise();
            if (p1.getOperator() == Operator.OR) {
                found = i;
            }
        }
        if (found < 0) {
            return "One of the premises must be a disjunction";
        }
        if (!conclusion.hasSubExpression(p1))
            return "the premises is not a disjunct in the conclusion";
        return null;
    }
}