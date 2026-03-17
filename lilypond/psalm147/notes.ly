melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 d2 a'2 b2 a4 b4 d4 cis4 b2 a2 r2 b2 b4 c4 d2 b2 a4 g4 f2 e2 r2 \break

  % Line 2
  e2 f4 g4 a2 f2 g4 a4 b2 a2 r2 \break

  % Line 3
  b2 d4 c4 b2 a2 f4 a4 g2 f2 r2 \break

  % Line 4
  f2 e4 d4 a'2 f2 g4 f4 e2 d2 r2 \break

  % Line 5
  a2 f4 b4 a4 f4 g4 a4 b2 a2 r2 \break

  % Line 6
  a2 d2 b2 c4 a4 b4 d4 c2 b2 r2 \break

  % Line 7
  d2 b4 b4 a4 f4 a4 a4 g2 f1 \bar "|."
}
